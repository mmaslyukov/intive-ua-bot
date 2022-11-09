# Guardian bot


## Embedded Target NanoPI aarch64

### Build image
```bash
docker build . -t guardian:latest
```

### Build `guardian` application
```bash
docker run --rm -v $(pwd):/app  guardian:latest
```

### Deployment
```bash
scp ./target/aarch64-unknown-linux-gnu/debug/guardian <username>@<IP>:/tmp
```