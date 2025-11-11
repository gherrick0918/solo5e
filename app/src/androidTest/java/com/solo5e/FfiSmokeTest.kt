package com.solo5e

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class FfiSmokeTest {
    @Test
    fun loadsAndCalls() {
        val v = Ffi.version()
        val sum = Ffi.roll(seed = 42L, n = 3, sides = 6)
        val len = Ffi.echoJsonLen("""{"hello":"world"}""")
        assertTrue(v.startsWith("solo5e-ffi"))
        assertTrue(sum in 3..18)
        assertTrue(len >= 14)
    }
}
