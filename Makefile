.PHONY: dev
dev:
	flutter run

.PHONY: depcheck
depcheck:
	dart_depcheck

.PHONY: lint
lint:
	flutter analyze

.PHONY: fmt
fmt:
	dart format .

.PHONY: clean
clean:
	flutter clean
