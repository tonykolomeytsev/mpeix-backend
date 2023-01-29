SELECT * FROM schedule_search_results
WHERE UPPER(name) LIKE UPPER('%$1%')
LIMIT 30;
