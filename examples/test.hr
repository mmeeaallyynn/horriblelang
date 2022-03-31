
/* Some useful stuff */
std {
	/* Consumes the value and jumps to a user defined "iftrue" or "else" */
	if is
		dup
		@iftrue jump?
		not @else jump?
	in

	/* Prints a newline */
	endl is
		"\n" print
	in
}

/* This could be how a struct looks like */
someStruct is
	a is 5 in
	b is 100 in
in

/* Do various things */
main is
	a is 1 in

	@main print \n print

	@a jump print \n print

	a is 2 in
	@a jump print \n print
	STACK
in

/* call main */
@main!

"meow:\n" print
@main::a! print

@std::endl!
@std::endl!

fn is
	"hello world!\n" print
in

/* test if/else*/
1

iftrue {
	"true\n" print
}
else {
	"false\n" print
} @std::if!

/* test boolean */
"Should be 0: " print
0 print @std::endl!

"Should be 1: " print
0 not print @std::endl!

"Should be 5: " print
5 print @std::endl!

"Should be 0: " print
5 not print @std::endl!


2 3 - print @std::endl!


/* Put Test */
a is 6 in
b is 100 in

@a! print @std::endl!
@b! print @std::endl!


5 @a put
@a! print @std::endl!
@b! print @std::endl!
STACK