create table users(
    id uuid primary key,
    username text not null unique,
    password text not null,
    nickname text not null,
    avatar_url text not null,
    created_at timestamp with time zone not null,
    updated_at timestamp with time zone not null
);

create table humans(
    id uuid primary key,
    name text not null
);

create table human_friends(
    human_id uuid references humans(id),
    friend_id uuid references humans(id),
    primary key(human_id, friend_id)
);
