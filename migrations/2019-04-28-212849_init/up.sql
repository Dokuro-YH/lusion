create table humans(
  id uuid primary key,
  name text not null
);

create table human_friends(
  human_id uuid references humans(id),
  friend_id uuid references humans(id),
  primary key(human_id, friend_id)
);
