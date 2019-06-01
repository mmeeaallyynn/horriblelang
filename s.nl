/* Some useful stuff */
s {
	/* Consumes the value and jumps to a user defined "then" */
	? is
		@=> jump?
	in

	/* Prints a newline */
	rn is
		"\n" print
	in

	pl is
		print @s::rn!
	in

	-- is
		ref is 0 in @ref put
		@ref! jump 1 - @ref! put
	in

	++ is
		ref is 0 in @ref put
		@ref! jump 1 + @ref! put
	in
}