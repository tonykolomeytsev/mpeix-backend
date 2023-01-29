SELECT * FROM schedule_search_results
WHERE UPPER(name) LIKE UPPER('%$1%') AND type='$2'
LIMIT 30;
