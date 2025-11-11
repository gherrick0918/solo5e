package com.solo5e

object Ffi {
    init {
        System.loadLibrary("ffi")
    }

    external fun version(): String
    external fun roll(seed: Long, n: Int, sides: Int): Int
    external fun echoJsonLen(json: String): Int
}
