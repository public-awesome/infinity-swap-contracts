.PHONY: optimize lint schema deploy-local e2etest e2etest-full

optimize:
	mkdir -p artifacts target
	sh scripts/optimize.sh

lint:
	cargo clippy --all-targets -- -D warnings

schema:
	sh scripts/schema.sh

deploy-local:
	docker kill stargaze || true
	docker volume rm -f stargaze_data
	docker run --rm -d --name stargaze \
		-e DENOM=ustars \
		-e CHAINID=testing \
		-e GAS_LIMIT=$(GAS_LIMIT) \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=stargaze_data,target=/root \
		publicawesome/stargaze:8.0.0 /data/entry-point.sh $(TEST_ADDRS)

e2etest:
	RUST_LOG=info CONFIG=configs/cosm-orc.yaml cargo integration-test $(TEST_NAME)

e2etest-full: deploy-local optimize e2etest
