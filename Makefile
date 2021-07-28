IMAGE_REPO = surfing-the-tide/apps:local

build:
	docker build \
		--progress=plain \
		--build-arg STRIP=1 \
		--build-arg DU_BUST=$(shell date +%s.%N) \
		--file Dockerfile \
		-t $(IMAGE_REPO) \
		$(PWD)

checks:
	cargo check
	cargo fmt
	cargo clippy

shell:
	docker run --rm -ti $(IMAGE_REPO) /bin/sh -l

start: dc.up

stop: dc.down

restart: stop start

dc.build:
	docker-compose build --force-rm --build-arg DU_BUST=$(shell date +%s.%N)

dc.up:
	docker-compose up -d --remove-orphans
	@echo "Open http://localhost:16686/ for jaeger"
	@echo "Call http://localhost:4000/ for front-server"

dc.down:
	docker-compose down --remove-orphans --volumes

dc.logs:
	docker-compose logs

wrk:
	wrk -c 20 -t 20 -R 20 -d 60 -L http://localhost:8080/

k6:
	k6 run k6.js

stats:
	docker stats --format="table {{.Name}}\t{{.CPUPerc}}\t{{.MemPerc}}\t{{.MemUsage}}" || exit 0
