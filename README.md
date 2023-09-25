# Cloudflare DDNS

This application can be used to periodically check for IP changes and update Cloudflare DNS records accordingly.

## Configuration

You will need to supply the following environment variables:
- `RUST_LOG`: The log level for the application. Valid values are `trace`, `debug`, `info`, `warn`, `error`
- `CLOUDFLARE_API_KEY`: Your Cloudflare API key
- `CLOUDFLARE_EMAIL`: The email address associated with your Cloudflare account
- `CLOUDFLARE_ZONE_ID`: The ID of the zone you want to monitor

## Usage

### Running the binary

```shell
export RUST_LOG=info
export CLOUDFLARE_EMAIL=example@gmail.com
export CLOUDFLARE_API_KEY=9351d89f6b1gdc2c0b0a5ef8c75d28eb8bf96
export CLOUDFLARE_ZONE=f681de33cd6c1eb8eb1bb7e8bd801222

cargo run --release
```

### Running the docker image
```shell
docker run jofish89/cloudflare-ddns:latest \
    -e RUST_LOG=info \
    -e CLOUDFLARE_EMAIL=example@gmail.com \
    -e CLOUDFLARE_API_KEY=9351d89f6b1gdc2c0b0a5ef8c75d28eb8bf96 \
    -e CLOUDFLARE_ZONE=f681de33cd6c1eb8eb1bb7e8bd801222
```

### Running in Kubernetes

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cloudflare-ddns-job
spec:
  schedule: "0 * * * *"  # Hourly
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: cloudflare-ddns
              image: jofish89/cloudflare-ddns:latest
              env:
                - name: RUST_LOG
                  value: "info"
                - name: CLOUDFLARE_EMAIL
                  value: "example@gmail.com"
                - name: CLOUDFLARE_API_KEY
                  value: "9351d89f6b1gdc2c0b0a5ef8c75d28eb8bf96"
                - name: CLOUDFLARE_ZONE
                  value: "f681de33cd6c1eb8eb1bb7e8bd801222"
          restartPolicy: OnFailure
```