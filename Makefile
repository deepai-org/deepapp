.PHONY: docker-build docker-test docker-check docker-build-example docker-migrate-preview docker-run

docker-build:
	docker compose build compiler

docker-test:
	docker compose run --rm test

docker-check:
	docker compose run --rm compiler check examples/chat.deep

docker-build-example:
	docker compose run --rm compiler build examples/chat.deep --out build

docker-migrate-preview:
	docker compose run --rm compiler migrate examples/chat.deep --preview

docker-run:
	docker compose up runtime
