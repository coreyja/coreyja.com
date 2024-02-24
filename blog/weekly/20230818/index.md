---
title: August 18th Update -- Rust Traits are just fancy Ruby Duck-typing
author: Corey Alexander
date: 2023-08-18
is_newsletter: true
---

## This Week

### Leaving Google Domains

Today I finally bit the bullet and transferred all my domains out of Google Domains and into Porkbun.

Google Domains was my go to for a couple years, but recently Google announced that they were selling the whole domain business to SquareSpace.

I don't have any specific problem with SquareSpace, but I was already trying to use Google Domains less. And I tend to use more technical tools than SquareSpace, nothing wrong with them just different tools for different people.

So today I transferred all my domains to Porkbun!
And it was way quicker and easier than I was expecting.

From start to finish took maybe 30-45 minutes and all my domains were in my Porkbun account. Wow! Both transfer pages warned it could take up to 5 days, so I was expecting a long process. But it seems like Google Domains and Porkbun have it all figured out. My only complaint was that there wasn't any bulk transfer out tools on the Google Domains side. I had ~10 domains to transfer over so wasn't awful to do one by one, but would have been nice to do all at once.

Feels good to have that taken care of. Now I don't have to worry about transfer to SquareSpace and even better I finally have all my domains in the same registrar!

### RustConf Talk Review

I'm speaking at RustConf next month, and I'm super excited for my talk titled "Using Rust and Battlesnake to never stop Learning".

And this week I had the awesome chance to do a run-through and get feedback from Sage Griffin from the Rust Foundation, which was great! I gave my talk and they provide feedback on how to make it even better for all of you. I've been a fan of Sage's for awhile and they gave me some really good feedback so it was really fun. I'm excited to polish up my talk and give it live in just under a month!

## Corey's Ramble

### Rust Traits are just fancy Ruby Duck-typing

Duck-typing is one of the things I initially loved Ruby for. Duck typing is when you have two different objects that have the same interface, such that you can pass either to some function and expect it to work. The name comes from phrase "If it quacks like a duck, then it's a duck". If two objects have the same methods, that can be treated as the same.

Duck-typing isn't a concept that's specific to Ruby. Most other languages call them an interface. Ruby's "twist" is that the interface isn't specified anywhere. Which of course is both the benefit and cost of dynamic languages like Ruby. Fans say it lets them move quickly while those in opposition might find it hard to maintain in large code bases. Both sides have their merit.

But one thing that I always loved about duck-typing was that I could pass in my _own_ objects instead of what a library was built to expect. As long as my object had the same methods, everything would #justwork. This flexibility allowed me to customize how libraries would interact, and something I find very powerful.

So when I started writing Rust I immediately took to trait. Traits are Rust interfaces, and fill the same space as duck-typing in Ruby. Really I think of traits as fancy duck-typing.

In rust I might write a function signature like the following

```rust
pub fn<T: Runable> do_some_work(job: T) {
 job.run();
}
```

This function takes in a `job`, which must implement the trait `Runnable`.

We don't know exactly what type `T` will be when we run this function, we just know it will implement Runable.

And similarly to Ruby duck-typing, I can make my own struct. And as long as I implement the `Runable` trait, I can pass my new struct to `do_some_work`.

And even better, the compiler can help me make sure that I implement all the methods I need to satisfy the trait. In Ruby I have to do all that work myself. I read the code I'm trying to pass my duck into, and see what methods it calls on our object. But this is time consuming and error prone. And even worse you have to remember to update all your "ducks" when you call a new method on the objects. This is even harder when you are trying to pass your "duck" to some external code, that is outside your control.
Rust requires us to specify the trait up front. Our `Runable` trait might look something like this

```rust
trait Runable {
 fn run(self);
}
```

This simply lists the functions that exist, with their full type signatures. Now rust can make sure everyone that implements this trait has a `run` method that consumes the type. And if we add something to our trait

```rust
trait Runable {
 fn run(self);

 fn run_later(self);
}
```

The compiler will make sure that each type that tries to implement `Runable` implements this new method too! Because of this we never have to worry about one type not fully implementing all the methods. In Ruby we'd get a "Method Not Found" error at Runtime; Rust stops us at compile time.

These concepts aren't unique to Ruby or Rust. Interfaces like this are common in most languages in one form or another. But It's fun to compare languages that are very different and see what tools they provide to solve similar problems.

Thanks for reading and see ya next week!
