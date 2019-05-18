"std.nl" include

fibonacci is
	counter is 0 in
		@counter put

	a is 0 in
	b is 1 in
	result is 0 in

	loop {
		@a! @b! + @result put

		@b! @a put
		@result! @b put

		@counter @std::dec!

		@counter! 0 > then {
			@loop!
		} @std::if!
	} @loop!

	@result!
in

60 @fibonacci! @std::println!
