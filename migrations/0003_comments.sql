
create table if not exists comments (
  id int unique generated always as identity,
  user_id int not null,
  post_id int not null,
  comment text not null,

  primary key (id),
  constraint fk_user
    foreign key (user_id)
      references users(id),
  constraint fk_post
    foreign key (post_id)
      references posts(id)
);

