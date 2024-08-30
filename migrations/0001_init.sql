
create table if not exists users (
    id int generated always as identity,
    username varchar(30) not null unique,
    email varchar(100) not null,
    primary key (id)
);

create table if not exists posts (
  id int unique generated always as identity,
  user_id int not null,
  title text not null,
  content text not null,
  primary key (id),
  constraint fk_user
    foreign key (user_id)
      references users(id)   
);

create table if not exists sessions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL UNIQUE,
    session_id VARCHAR NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);