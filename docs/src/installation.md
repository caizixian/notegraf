# Docker Compose (recommended)
First, create a folder `notegraf`.
Then, within that folder, create a `docker-compose.yml` file.
```console
notegraf
└── notegraf_config.yml
```

```yaml
# docker-compose.yml
version: '3'
services:
  db:
    image: postgres:14
    restart: always
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
      POSTGRES_DB: notegraf
      LANG: C.UTF-8
    volumes:
      - dbdata:/var/lib/postgresql/data
  notegraf:
    image: ghcr.io/caizixian/notegraf:master
    restart: always
    depends_on:
      - "db"
    ports:
      - "8000:8000"
    environment:
      NOTEGRAF_HOST: "0.0.0.0"
      NOTEGRAF_PORT: 8000
      NOTEGRAF_NOTESTORETYPE: "PostgreSQL"
      NOTEGRAF_DATABASE_HOST: "db"
      NOTEGRAF_DATABASE_PORT: 5432
      NOTEGRAF_DATABASE_USERNAME: postgres
      NOTEGRAF_DATABASE_PASSWORD: password
      NOTEGRAF_DATABASE_NAME: notegraf
      NOTEGRAF_DEBUG: false
volumes:
  dbdata:
```

Within the folder, run `docker-compose up -d`.
Your Notegraf instance should be up and running.
Open <http://localhost:8000> in your browser and see for yourself. 

To update Notegraf, run `docker pull ghcr.io/caizixian/notegraf:master` and run `docker-compose up -d` again. 