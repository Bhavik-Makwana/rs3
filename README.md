# RS3

I work a lot of with databases, and I've been meaning to learn more about how they work.
The databases I usually work with are F1 and Spanner
https://static.googleusercontent.com/media/research.google.com/en//archive/spanner-osdi2012.pdf
https://static.googleusercontent.com/media/research.google.com/en//archive/spanner-osdi2012.pdf

These are both distributed databases and are very complex. So instead I wanna start with something simpler.

I'm following along with the tutorial at:
- https://cstack.github.io/db_tutorial/

I'm rewriting this in Rust as a learning exercise. Its a good tutorial but they have known gaps which I'll address in some cases.

Hopefully I can extend this to use some of the concepts from F1 and Spanner by expanding it into a distributed database.

I've previously implemented paxos in Rust which should provide a good foundation for this.