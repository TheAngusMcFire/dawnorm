mdebug:
	watch --color -n 1 -- 'cat /tmp/test.rs | rustfmt --edition 2021 | bat --color=always -l rust'

cdebug:
	cat /tmp/test.rs | rustfmt --edition 2021 | bat --color=always -l rust