create table users (
    userid int generated always as identity,
    username varchar(30) not null,
    email varchar(100) not null,
    primary key (userid)
);