.PHONY: optimize optimize-arm lint schema dl-launchpad-artifacts dl-marketplace-artifacts
.PHONY: dl-artifacts deploy-local e2e-test e2e-test-full e2e-test-full-arm

TEST_ADDRS ?= $(shell jq -r '.[].address' ./e2e/configs/test_accounts.json | tr '\n' ' ')
GAS_LIMIT ?= "75000000"

artifacts:
	mkdir artifacts

target:
	mkdir target

optimize: artifacts target
	sh scripts/optimize.sh

optimize-arm: artifacts target
	sh scripts/optimize-arm.sh

dl-launchpad-artifacts: artifacts
	scripts/dl-launchpad-artifacts.sh

dl-marketplace-artifacts: artifacts
	scripts/dl-marketplace-artifacts.sh

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

dl-artifacts: dl-launchpad-artifacts dl-marketplace-artifacts

e2e-test: deploy-local
	RUST_LOG=info CONFIG=configs/cosm-orc.yaml cargo integration-test $(TEST_NAME)

e2e-test-full: dl-artifacts optimize e2e-test

e2e-test-full-arm: dl-artifacts optimize-arm e2e-test
