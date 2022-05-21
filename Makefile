publish:
	cd $(shell pwd)/hermes && cargo build --release
	sudo systemctl stop olympos.hermes
	sudo cp $(shell pwd)/hermes/target/release/hermes /usr/share/olympos/hermes/
	sudo systemctl start olympos.hermes
	sudo rm -r /usr/share/olympos/hermes/agents/*
	sudo cp $(shell pwd)/agents/* /usr/share/olympos/hermes/agents/

publish_agents:
	sudo rm -r /usr/share/olympos/hermes/agents/*
	sudo cp $(shell pwd)/agents/* /usr/share/olympos/hermes/agents/

publish_hermes:
	cd $(shell pwd)/hermes && cargo build --release
	sudo systemctl stop olympos.hermes
	sudo cp $(shell pwd)/hermes/target/release/hermes /usr/share/olympos/hermes/
	sudo systemctl start olympos.hermes

