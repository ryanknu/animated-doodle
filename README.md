# Chat Sample

## Instructions

Out of the box, this uses ports 5050 and 3000, so it's helpful to make sure
nothing is running on those ports locally. If you have a conflict that you
can't resolve, simply edit the docker-compose.yml and replace these ports:

```
sed -i s|5050|8888|g docker-compose.yml
sed -i s|localhost|10.4.4.4|g docker-compose.yml
```

After that, we need to install npm packages. I have tried to make this happen
a few ways automatically on docker start, by modifying the command and rolling
a custom docker-entrypoint.sh, but I couldn't seem to make it happy doing those
things. What worked was using a dockerfile to build the stack, but that's for
deployment, not development, so I opted to not do that. So run:

```
pushd web; npm i; popd
```

After that, to bring up the stack, to run the stack:

```
docker compose up
```

Feel free to freshen your tea. Rust isn't known for fast initial compile time.

The after this the web app should be available at http://localhost:3000/.

## Organization

The app is ogranized into front end and back end:

- api: the back end. Exposes an HTTP API written in Rust using the tokio
  ecosystem (axum, tower, hyper). The data storage layer is DynamoDB. Rust was
  chosen becuase it hides nothing from the user, even copying a pointer to pass
  to another thread needs to be explicitly done, and you can't loop through the
  same range of bytes twice without declaring them mutable. Because of this the
  reader of the code has an incredibly good sense of what is going on. Also, I
  just like it.
- web: the front end. I chose to use a Next app for this for incredibly fast
  compile times, pretty sane defaults, and zero config routing.

## Conventions

The following conventions are helpful to know to help navigate the code:

### Web

API communication happens in hooks. Instead of exposing one API client, subsets
of features are broken out into related modules, for example, status checker,
chat room listing, and message sending. Using hooks exposes the page-level
components to an API that feels realtime.

Asynchronous actions are tracked with a variable called `workingError` which is
semantic for "is working or error message". It's type is `string|boolean`, and
it's states can be interpreted as such:

- `true` - API communication is happening
- `false` - API communication isn't happening, and no messages should be shown
- `(some string)` - An error message should be shown to the user.

### API

`ChatError` is the error type. It `impl IntoResponse` so basically, any method
that returns an error, can instead `?` a `ChatError` to return a 500. This
sort of makes all back end error handling evaporate into a sea of `?`. Whenever
an option is returned (`Some` or `None`) I prefer to use `ok_or_else` to
convert `None` into a `ChatError`, then `?` to return it.

The way the aws_sdk_rust is set up is that it's methods map 1:1 with the XML
API. The constructs and enums provided create a set of nested builders that
generate a structure that matches the XML API.

This is a RESTful API. This means that all operations return objects with a
uniqude identifier that is a URI in them. This format is well-suited to
JSON-LD, so that's what I used, but I did not provide schema files.

## Data Structure

The app uses DynamoDB so that all operations complete in constant time. No single
request performs more than 3 operations, meaning, the maximum time spent in DynamoDB
per request is 45ms. This can't be helped locally, but in the could you could deploy
DAX to bring this down to a maximum of 4.5 seconds.

### "messages" Table

Sort Key: (room_id N HASH, sort S RANGE)
GSI: "name-index": (name S HASH) [used to check if a room exists by name]

Note: `sort = active_rooms` is special and will only exist with `room_id = 1`, and it
holds the stack of most recently used room ID's.

| room_id | sort         | room_ids | name          | sender_id | sender_name | message |
| ------- | ------------ | -------- | ------------- | --------- | ----------- | ------- |
| 1       | active_rooms | 73,19    |               |           |             |         |
| 73      | room         |          | Bird Watching |           |             |         |
| 73      | message.1    |          |               | 66        | Ryan        | Hello   |
| 73      | message.2    |          |               | 50        | Peter       | He-haw  |
| 19      | room         |          | Fan club      |           |             |         |
| 19      | message.3    |          |               | 66        | Ryan        | Hello   |

### "users" Table

Sort Key: (user_id N HASH)
GSI: "name-index": (name S HASH) [used to check if a user exists by name]

| user_id | name  |
| ------- | ----- |
| 66      | Ryan  |
| 50      | Peter |
