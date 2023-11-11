watch_print:
	cargo watch -c -- make cdebug

cdebug:
	cat /tmp/test.rs | rustfmt --edition 2021 | bat --color=always -l rust