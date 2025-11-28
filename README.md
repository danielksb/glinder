# Glinder

A dating web site for lonely gloves.


## Features

- Swipe through a collection of lost gloves to find the right match.
- Simple admin upload UI to provide new profiles (protected by Basic auth; credentials provided through Spin variables)

## Development

1. Add the wasm32-wasip1 Rust target if you haven't already:

```shell
rustup target add wasm32-wasip1
```

2. Build:

```shell
spin build
```

3. Start the Spin application locally.

```shell
$env:SPIN_VARIABLE_USERNAME = 'admin'; $env:SPIN_VARIABLE_PASSWORD = 'secret'; spin up
```

After `spin up`, the app will be available at http://127.0.0.1:3000/.
