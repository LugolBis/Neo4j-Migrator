# Example of Queries on *Nothwind* database

## Objective ?

This document present you example of **SQL** queries on *Northwind* database, translated into **Cypher** queries.

It might be very useful for you to understand how ***Neo4j-Migrator*** transforms the relational database into a graph database.

## Queries in SQL | Cypher
> /!\ WARNING <br>
> The Cypher queries following the ***Neo4j-Migrator*** normalisation. It's normal if the **foreign key** of your relationnal model are not store in the nodes.<br>
> Read the **Query 3** and **Query 4**, will help you to understant that.

### Query 1
SQL :
```SQL
SELECT category_name, Description
FROM categories;
```
Cypher :
```CYPHER
match (n:CATEGORIES)
return n.description, n.category_name;
```

### Query 2
SQL :
```SQL
SELECT UPPER(first_name) AS first_name, UPPER(last_name) AS last_name, hire_date
FROM employees
ORDER BY hire_date
```
Cypher :
```CYPHER
match (n:EMPLOYEES)
return UPPER(n.first_name) as first_name, UPPER(n.last_name) as last_name, n.hire_date
order by n.hire_date;
```

### Query 3
SQL :
```SQL
SELECT order_id,OrderDate,shipped_date,customer_id,freight
FROM orders
ORDER BY freight Desc
LIMIT 10;
```
Cypher :
```CYPHER
match (o:ORDERS)-[r]-(c:CUSTOMERS)
return o.order_id, o.order_date, o.shipped_date, c.customer_id, o.freight
order by o.freight desc
limit 10;
```
Or
```CYPHER
match (o:ORDERS)-[r:ORDERS__REF__CUSTOMER_ID]-(c)
return o.order_id, o.order_date, o.shipped_date, c.customer_id, o.freight
order by o.freight desc
limit 10;
```

### Qyery 4
SQL :
```SQL
SELECT company_name,contact_name
FROM customers
WHERE city='BuenosAires';
```
Cypher :
```Cypher
match (c:CUSTOMERS)
where c.city = "Buenos Aires"
return c.company_name, c.contact_name;
```

### Query 5
SQL :
```SQL
SELECT contact_name,Address,city
FROM customers
WHERE country NOT IN ("Germany","Mexico","Spain");
```
Cypher :
```Cypher
match (c:CUSTOMERS)
where not c.country in ["Germany","Mexico","Spain"]
return c.contact_name, c.address, c.city;
```

### Query 6
SQL :
```SQL
SELECT first_name,last_name,country
FROM employees
WHERE country != 'USA';
```
Cypher :
```Cypher
match (e:EMPLOYEES)
where e.country <> "USA"
return e.first_name, e.last_name, e.country;
```

### Query 7
SQL :
```SQL
SELECT employee_id,order_id,customer_id,required_date,shipped_date
FROM orders
WHERE shipped_date>required_date;
```
Cypher :
```Cypher
match (c:CUSTOMERS)-[r1]-(o:ORDERS)-[r2]-(e:EMPLOYEES)
where o.shipped_date > o.required_date
return e.employee_id, o.order_id, c.customer_id, o.required_date, o.shipped_date;
```

### Query 7
SQL :
```SQL
SELECT city,company_name,contact_name
FROM customers
WHERE city LIKE "A%"OR city LIKE "B%";
```
Cypher :
```Cypher
match (c:CUSTOMERS)
where c.city starts with "A"
or c.city starts with "B"
return c.city, c.company_name, c.contact_name;
```

### Query 8
SQL :
```SQL
SELECT order_id
FROM orders
WHERE mod(order_id, 2) = 0;
```
Cypher :
```Cypher
match (o:ORDERS)
where o.order_id%2 = 0
return o.order_id;
```

### Query 9
SQL :
```SQL
SELECT o.order_id, count(o.order_id) as NumberOfOrders
FROM order_details o
GROUP BY o.order_id
ORDER BY NumberOfOrders DESC;
```
Cypher :
```Cypher
match (od:ORDER_DETAILS)-[r]-(o:ORDERS)
WITH o.order_id as ID, count(od) as NumberOfOrders
return ID, NumberOfOrders
order by NumberOfOrders desc;
```

### Query 10
SQL :
```SQL
SELECT s.supplier_id,p.product_name,s.company_name
FROM suppliers s
JOIN products p ON s.supplier_id=p.supplier_id
WHERE s.company_name IN('Exotic Liquids','Specialty Biscuits, Ltd.','Escargots Nouveaux')
ORDER BY s.supplier_id;
```
Cypher :
```Cypher
match (s:SUPPLIERS)-[r]-(p:PRODUCTS)
where s.company_name in ['Exotic Liquids','Specialty Biscuits, Ltd.','Escargots Nouveaux']
return s.supplier_id,p.product_name,s.company_name
order by s.supplier_id;
```

### Query 11
SQL :
```SQL
SELECT CONCAT( first_name,' ', last_name ,' can be reached at ', 'x',extension ) AS Contactinfo
FROM employees;
```
Cypher :
```Cypher
match (e:Employee)
return e.first_name + " " + e.last_name + " can be reached at x" + e.extension AS ContactInfo;
```

### Query 12
SQL :
```SQL
SELECT s.supplier_id,s.company_name,c.category_name,p.product_name,p.unit_price
FROM products p
JOIN suppliers s ON s.supplier_id = p.supplier_id
JOIN categories c On c.category_id=p.category_id;
```
Cypher :
```Cypher
match (s:SUPPLIERS)-[r1]-(p:PRODUCTS)-[r2]-(c:CATEGORIES)
return s.supplier_id,s.company_name,c.category_name,p.product_name,p.unit_price;
```

### Query 13
SQL :
```SQL
SELECT customer_id, sum(freight) as Total
FROM orders
GROUP BY customer_id
HAVING sum(freight)>200;
```
Cypher :
```Cypher
match (o:ORDERS)-[r]-(c:CUSTOMERS)
with c.customer_id as ID, sum(o.freight) as Total
where Total > 200
return ID, Total;
```

### Query 14
SQL :
```SQL
SELECT a.last_name as employee, b.last_name as manager
FROM employees a
LEFT JOIN employees b ON b.EmployeeID = a.ReportsTo;
```
Cypher :
```Cypher
match (a:EMPLOYEES)
optional match (a)-[:EMPLOYEES__REF__REPORTS_TO]->(b:EMPLOYEES)
return a.last_name as employee, b.last_name as manager;
```