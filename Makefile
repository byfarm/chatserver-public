default:
	espflash flash ./target/xtensa-esp32-espidf/debug/server --monitor

monitor:
	espflash monitor

serve:
	./server/target/release/chat

resetdb: 
	rm server/chat.db
	touch server/chat.db
	cat server/db.dump | sqlite3 server/chat.db
