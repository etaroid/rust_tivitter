CREATE USER IF NOT EXISTS 'user'@'%' IDENTIFIED BY 'password';
CREATE DATABASE IF NOT EXISTS `test` DEFAULT CHARACTER SET `utf8mb4` COLLATE `utf8mb4_bin`;
CREATE DATABASE IF NOT EXISTS `production` DEFAULT CHARACTER SET `utf8mb4` COLLATE `utf8mb4_bin`;
GRANT ALL PRIVILEGES ON test.* TO 'user'@'%';
GRANT ALL PRIVILEGES ON production.* TO 'user'@'%';
ALTER USER root IDENTIFIED WITH mysql_native_password BY 'password';