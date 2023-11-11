watch_print:
	cat /tmp/test.rs | rustfmt --edition 2021 | bat --color=always -l rust