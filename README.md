## Outbreak: Origins Server

### Setting Up

- Install postgresql
- Create your user and change the .env file to reflect that
- Make sure the user has the CREATEDB permission
- Create the 'outbreak-origins' database
- Run `diesel migration run`

### Dev

To run the server in watch mode for auto-reloading, install cargo-watch with ```cargo install cargo-watch``` or with a distro-specific method and run<br>
```cargo watch -x run```
