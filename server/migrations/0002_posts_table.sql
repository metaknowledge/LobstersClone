create table posts (
  postid int unique generated always as identity,
  userid int not null,
  title text not null,
  content text not null,
  primary key (postid),
  constraint fk_user
    foreign key (userid)
      references users(userid)   
);


