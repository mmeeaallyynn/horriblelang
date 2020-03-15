"std.nl" include

structure is
Vector
	a is 5 in
	b is 6 in
in

something_else is
Vector
	a is 100 in
	b is 200 in
in

one_more_thing is
NotAVector
	f is 5 in
in

do_stuff is
	jump Vector != then {
		"error: argument is not a vector!"
	} @std::if!

	@a! @b! +
in

STACK

@structure
	@do_stuff!
	@std::println!

STACK

@something_else
	@do_stuff!
	@std::println!

STACK

@one_more_thing
	@do_stuff!
	@std::println!

STACK