package com.solo5e

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class FfiSmokeTest {
    @Test fun loadsAndCalls() {
        val v = Ffi.version()
        assertTrue(v.startsWith("solo5e-ffi"))

        val sum = Ffi.roll(seed = 42L, n = 3, sides = 6)
        assertTrue(sum in 3..18)

        val len = Ffi.echoJsonLen("""{"hello":"world"}""")
        assertTrue(len >= 14)
    }
}
