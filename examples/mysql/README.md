# TODOs Example

## Usage

```sh
docker run --rm -p 3306:3306 \
    -e MYSQL_ROOT_PASSWORD=root \
    -e MYSQL_DATABASE=examples \
    -e MYSQL_USER=user \
    -e MYSQL_PASSWORD=password \
    mysql:9
cargo run --example mysql --features mysql
```
