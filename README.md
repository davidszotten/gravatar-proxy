# Gravatar-proxy

Gravatar proxy to improve privacy. Uses [fernet](https://github.com/fernet/spec) for secret sharing.

## Usage

```
Gravatar proxy

USAGE:
    gravatar-proxy [OPTIONS] <KEY>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bind <ADDR>    Bind to a specific address (ip:port) [default: localhost:6000]

ARGS:
    <KEY>    Password for encrypting the email addresses
```

Serves images at `http://<ADDR>/avatar/<encrypted>`, where `encrypted` is the **email address** encrypted using the shared Fernet key. Any query params are passed along unchanged.

## License

MIT
