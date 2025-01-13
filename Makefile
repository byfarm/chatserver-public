default:
	espflash flash ./target/xtensa-esp32-espidf/debug/server --monitor

monitor:
	espflash monitor

serve:
    ./server/target/release/chat

resetdb:
    rm chat.db
    touch chat.db
    cat server/db.dump | sqlite3 chat.db
