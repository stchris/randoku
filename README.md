# randoku

[![Tests & Clippy](https://github.com/stchris/randoku/actions/workflows/test.yml/badge.svg)](https://github.com/stchris/randoku/actions/workflows/test.yml)


a service which generates random numbers so you don't have to

See it in action here: https://randoku.shuttleapp.rs

### Usage

The index page at https://randoku.shuttleapp.rs shows an API overview page when called from a browser, but responds with a random number between 0 and 100 otherwise. This is based on a user-agent check. Numbers passed as parameters need to be between 0 and 18446744073709551615.

Request a random number between 0 and 100:
```
$ curl https://randoku.shuttleapp.rs/
4
```

Request a random number between 0 and a given number:
```
$ curl https://randoku.shuttleapp.rs/42
4
```

Request a random number between a given interval:
```
$ curl https://randoku.shuttleapp.rs/4/42
42
```

Shuffle a list of comma-separated items:
```
$ curl https://randoku.shuttleapp.rs/shuffle/apples,bananas,oranges
bananas
oranges
apples
```

### Running locally

```
$ cargo shuttle run
```
