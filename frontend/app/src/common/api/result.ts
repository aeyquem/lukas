export type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };

export type Maybe<T> = { kind: "some"; value: T } | { kind: "none" };
