"std.nl" include

my-ref is _ in
my-first-ref is _ in

11 @memory::alloc! -> @my-first-ref
5 @memory::alloc! -> @my-ref
7 @memory::alloc! drop
//@memory::block-list! STACK
//@memory::free-list! STACK

@my-ref$ @memory::free!
@my-first-ref$ @memory::free!


@memory::dump-free-list!

//ref2: print \space print 11 @memory::alloc! print  \n print

//| @memory::free-list! lambda print \space print in @lists::foreach! \n print

