SELECT SUM(size) FROM (SELECT size FROM allocations ORDER BY size DESC LIMIT 3)
SELECT SUM(size), COUNT(*) FROM (SELECT size FROM allocations WHERE callstack LIKE '%linear%' ORDER BY size LIMIT 1200)
SELECT COUNT(*) FROM allocations WHERE callstack LIKE '%linear%' ORDER BY size