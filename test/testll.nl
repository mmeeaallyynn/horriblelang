linkedlist.nl include

ll1 is _ in
ll2 is _ in
ll3 is _ in
ll4 is _ in

0 @linked-list::new! -> @ll3
1 20 @lists::range! lambda @ll3$ @linked-list::push! in @lists::foreach!

1 @linked-list::new! -> @ll4

5 @linked-list::new! -> @ll1
3 @linked-list::new! -> @ll2
10 @ll1$ @linked-list::push!
4 @ll2$ @linked-list::push!
11 @ll1$ @linked-list::push!
5 @ll2$ @linked-list::push!
17 @ll1$ @linked-list::push!
6 @ll2$ @linked-list::push!
7 @ll2$ @linked-list::push!
8 @ll2$ @linked-list::push!
9 @ll2$ @linked-list::push!
10 @ll2$ @linked-list::push!


@ll1$ @linked-list::dump!
length: print \space print @ll1$ @linked-list::len! @std::println!

@ll2$ @linked-list::dump!
length: print \space print @ll2$ @linked-list::len! @std::println!

@ll3$ @linked-list::dump!
length: print \space print @ll3$ @linked-list::len! @std::println!

@ll3$ @linked-list::pop! @std::println!
@ll3$ @linked-list::pop! @std::println!
@ll3$ @linked-list::pop! @std::println!
@ll3$ @linked-list::pop! @std::println!

@ll3$ @linked-list::dump!
length: print \space print @ll3$ @linked-list::len! @std::println!

1 @ll1$ @linked-list::at! @std::println!
