from transformers import AutoModelForTokenClassification
from razdel import tokenize
import torch
from transformers import pipeline
from transformers import AutoTokenizer
import re
from natasha import Doc, MorphVocab, NewsMorphTagger, NewsEmbedding, Segmenter
from natasha.norm import normalize


def deEmojify(text):
    regrex_pattern = re.compile(
        pattern="["
        "\U0001F600-\U0001F64F"  # emoticons
        "\U0001F300-\U0001F5FF"  # symbols & pictographs
        "\U0001F680-\U0001F6FF"  # transport & map symbols
        "\U0001F1E0-\U0001F1FF"  # flags (iOS)
        "\U0001F914"
        "]+",
        flags=re.UNICODE,
    )
    return regrex_pattern.sub(r"", text)


def removelinks(text):
    return re.sub(f"https://.* ", "", text)


def removehashtags(text):
    return re.sub(f"#.*", "", text)


def standartize(text):
    return re.sub(f"[«»“”]", '"', text)


class Model:
    def __init__(self, path_to_weights="./weights.pt"):
        print(f"Path to weights: {path_to_weights}")
        model_checkpoint = "DeepPavlov/rubert-base-cased"
        self.label_list = [
            "O",
            "B-date",
            "B-event",
            "B-name",
            "B-place",
            "B-subtitle",
            "B-time",
            "I-date",
            "I-event",
            "I-name",
            "I-place",
            "I-subtitle",
            "I-time",
        ]

        self.model = AutoModelForTokenClassification.from_pretrained(
            model_checkpoint, num_labels=len(self.label_list)
        )
        self.model.config.id2label = dict(enumerate(self.label_list))
        self.model.config.label2id = {
            v: k for k, v in self.model.config.id2label.items()
        }
        self.model.load_state_dict(
            torch.load(path_to_weights, map_location=torch.device("cpu"))
        )
        self.tokenizer = AutoTokenizer.from_pretrained(model_checkpoint)
        self.segmenter = Segmenter()
        self.emb = NewsEmbedding()
        self.morph_tagger = NewsMorphTagger(self.emb)
        self.morph_vocab = MorphVocab()
        self.exlusions = ["ДК", "МЭИ", "БАЗ"]

    def clean_text(self, text):
        text = deEmojify(text)
        text = removelinks(text)
        text = removehashtags(text)
        text = standartize(text)
        return text

    def form_event_name(self, subtitle, event, name):
        subtitle = "" if not subtitle else subtitle[0]
        name = "" if not name else name[0]
        event = sorted(event, key=lambda x: len(x.split()))
        event = "" if not event else event[0]
        if len(event.split()) > 1 or name:
            subtitle = ""
        if event != "" and (
            event.lower() in name.lower() or self.check_inclusion(event, subtitle)
        ):
            event = ""
        return " ".join([subtitle, event, name]).strip()

    def check_inclusion(self, event, subtitle):
        doc1 = Doc(event)
        doc2 = Doc(subtitle)
        doc1.segment(self.segmenter)
        doc2.segment(self.segmenter)
        doc1.tag_morph(self.morph_tagger)
        doc2.tag_morph(self.morph_tagger)
        for doc in [doc1, doc2]:
            for token in doc.tokens:
                token.lemmatize(self.morph_vocab)
        event = doc1.tokens[0].lemma
        if event in [_.lemma for _ in doc2.tokens]:
            return True
        else:
            return False

    def normalize(self, init_word, name):
        if name == "name":
            if init_word.startswith('"') and len(init_word) > 1:
                word = '"' + init_word[2].upper() + init_word[3:]
                return word
            else:
                return init_word
        doc = Doc(init_word)
        doc.segment(self.segmenter)
        doc.tag_morph(self.morph_tagger)
        #    print(doc.tokens)
        word = normalize(self.morph_vocab, doc.tokens).split()
        if name == "event":
            normalized_word = []
            mark = False
            last_pos = None
            complex_word = False
            for i, _ in enumerate(doc.tokens):
                # [ADJ]? [NOUN], [ADP(orig)], [ETC(orig)]
                # [ADJ]?-?[NOUN], [ADJ(orig)]? [NOUN(orig)]
                # [NOUN]-[NOUN], [ADJ(orig)] [NOUN(orig)]
                if "".join(normalized_word) == "Мастер-класс" or _.pos in (
                    "CCONJ",
                    "ADP",
                ):
                    mark = True
                if _.pos == "PUNCT":
                    if _.text == "-":
                        complex_word = True
                    wr = _.text
                elif _.text[:4] == "квиз":
                    wr = "квиз"

                elif complex_word:
                    sub_doc = Doc(_.text)
                    sub_doc.segment(self.segmenter)
                    sub_doc.tag_morph(self.morph_tagger)
                    wr = normalize(self.morph_vocab, sub_doc.tokens)
                    complex_word = False
                    mark = True
                elif mark:
                    wr = _.text
                elif i == 0:
                    wr = word[i].capitalize()
                elif last_pos in ("ADJ", "ADV") and i == 1:
                    wr = word[i]
                elif last_pos in ("PROPN", "NOUN", "VERB", "ADJ") and _.pos in (
                    "NOUN",
                    "PROPN",
                    "ADJ",
                ):
                    wr = _.text
                else:
                    break
                last_pos = _.pos

                normalized_word.append(wr)
            if last_pos in ("CCONJ", "ADP"):
                normalized_word.pop()
            return " ".join(normalized_word)
        if name == "place":
            word = normalize(self.morph_vocab, doc.tokens)
            doc_ = Doc(word)
            doc_.segment(self.segmenter)
            doc_.tag_morph(self.morph_tagger)
            new_word = []
            for i, token in enumerate(doc.tokens):
                if token.text in self.exlusions:
                    new_word.append(token.text)
                else:
                    new_word.append(doc_.tokens[i].text)
        return " ".join(new_word)

    def format(self, word, entity_name):
        if entity_name in ["event", "name", "place"]:
            word = self.normalize(word, entity_name)
        word = re.sub(r'" (?=.*")', '"', word)
        word = re.sub(r' "(?!.*")', '"', word)
        word = re.sub(r" , ", ", ", word)
        word = re.sub(r"(\( | \))", "", word)
        word = re.sub(f" …", "...", word)
        word = re.sub(r" - ", "-", word)
        if entity_name in ("name", "event"):
            word = re.sub(r" : ", ": ", word)
            word = re.sub(r"(?<! )\. ", ".", word)
        elif entity_name == "place":
            word = re.sub(r"(^в[о]* |^на )", "", word)
            word = re.sub(r"-го", "", word)
            word = re.sub(f" \. ", ". ", word)
        elif entity_name == "time":
            if ":" in word:
                word = word.split()
                word = ":".join([word[0], word[2]])
            else:
                word = re.sub(r"\. ", ":", word)
        elif entity_name == "subtitle":
            word = self.format_subtitle(word)
        elif entity_name == "date":
            word = re.sub(r" \. ", ".", word)
            word = re.sub(r"-го", "", word)
        else:
            raise AttributeError(f"Entity name {entity_name} is not allowed")
        return word

    def format_subtitle(self, word):
        if word.isupper():
            lower_word = word.lower().capitalize()
            new_subtitle = []
            old_subtitle = word.split(" ")
            for i in range(len(old_subtitle)):
                if (
                    "".join([j for j in old_subtitle[i] if j.isalnum()])
                    in self.exlusions
                ):
                    new_subtitle.append(old_subtitle[i])
                else:
                    new_subtitle.append(lower_word.split(" ")[i])
            word = " ".join(new_subtitle)
        return word

    def predict(self, text: str) -> dict[str, str]:
        pipe = pipeline(
            model=self.model,
            tokenizer=self.tokenizer,
            task="ner",
            aggregation_strategy="average",
            device=torch.device("cpu"),
        )
        cleaned_text = self.clean_text(text)
        pred = pipe(cleaned_text)
        #  print(pred)
        dikt = {
            "event": [],
            "name": [],
            "date": [],
            "time": [],
            "place": [],
            "subtitle": [],
        }
        last_idx = 0
        last_word = []
        last_entity = None
        for word in pred:
            if word["score"] > 0.1:
                if (last_entity == word["entity_group"]) and (
                    word["start"] < last_idx + 2
                ):  # or len(word['word'].split()[0])<=2)
                    last_word.append(word["word"])
                    last_idx = word["end"]
                else:
                    if last_entity:
                        phrase = self.format(" ".join(last_word), last_entity)
                        if last_entity == "name" and last_word[0] == '"':
                            names_in_text = re.findall(r'"[^"]*"', text)
                            for i in range(len(names_in_text)):
                                if phrase in names_in_text[i]:
                                    dikt["name"].append(names_in_text[i])
                        else:
                            dikt[last_entity].append(phrase)
                    last_entity = word["entity_group"]
                    last_idx = word["end"]
                    last_word = [word["word"]]
        phrase = self.format(" ".join(last_word), last_entity)
        if last_entity == "name" and last_word[0] == '"':
            names_in_text = re.findall(r'"[^"]*"', text)
            for i in range(len(names_in_text)):
                if phrase in names_in_text[i]:
                    dikt["name"].append(names_in_text[i])
        else:
            dikt[last_entity].append(phrase)
        event_name = self.form_event_name(dikt["subtitle"], dikt["event"], dikt["name"])
        #   print(dikt)
        place = "" if not dikt["place"] else dikt["place"][0]
        return {
            "name": event_name,
            "date": dikt["date"][0],
            "time": dikt["time"][0],
            "place": place,
        }
