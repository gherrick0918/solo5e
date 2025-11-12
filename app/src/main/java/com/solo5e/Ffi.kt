package com.solo5e

object Ffi {
    init { System.loadLibrary("ffi") } // loads libffi.so
    external fun version(): String
    external fun roll(seed: Long, n: Int, sides: Int): Int
    external fun echoJsonLen(json: String): Int
    external fun simulateDuelJson(json: String): String
    external fun simulateDuelManyJson(json: String): String
    external fun simulateEncounterJson(json: String): String
}
