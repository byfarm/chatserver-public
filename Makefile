default:
	espflash flash ./target/xtensa-esp32-espidf/debug/server --monitor

monitor:
	espflash monitor
