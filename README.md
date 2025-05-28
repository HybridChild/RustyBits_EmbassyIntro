# Intro to Embassy on stm32NUCLEO

This project implements the examples from the YouTube video [**Intro to Embassy: embedded development with async Rust**](https://youtu.be/pDd5mXBF4tY?si=SiOeY5AuNph4R-Cl) by **TheRustyBits** on the stm32NUCLEO-F072RB Development Board.

## Notes from video
An async function is effectively a constructor for an unamed Future type. And that future is our unit of concurency, runnable by either:
- Awaiting it which will return control to the executor whenever it's Pending - `my_future.await`
- By polling it manually which will allow us to collect the result and decide how to proceed - `my_future.poll(...)`

The 2nd option is where combinators like `join` and `select` come into the picture.
- `join` will poll each future that it's given and only return Ready once they're all completed - `join(fut1, fut2, ...)`
- `select` also polls the provided futures but will return Ready as soon as the first completes, dropping the others. `select(fut1, fut2, ...)`

`join` and `select` are available as macros in the futures crate and also as functions within the embassy-futures crate.
