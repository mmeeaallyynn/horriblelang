/* Some useful stuff */
std {
	/* Consumes the value and jumps to a user defined "then" */
	if is
		@then jump?
	in

	/* Prints a newline */
	endl is
		"\n" print
	in

	println is
		print @std::endl!
	in

	dec is
		ref is 0 in @ref put
		@ref! jump 1 - @ref! put
	in

	inc is
		ref is 0 in @ref put
		@ref! jump 1 + @ref! put
	in
}
