package com.solo5e

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicInteger

@RunWith(AndroidJUnit4::class)
class FfiMoreTests {
    @Test fun determinismAndBounds() {
        val a = Ffi.roll(123L, 1000, 6)
        val b = Ffi.roll(123L, 1000, 6)
        assertEquals(a, b)
        assertTrue(a in 1000..6000)
        assertEquals(0, Ffi.roll(1L, 0, 6))
        assertEquals(5, Ffi.roll(1L, 5, 1)) // sides=1 → always 1
    }

    @Test fun unicodeRoundTrip() {
        val len = Ffi.echoJsonLen("""{"emoji":"☕️🔥🎸","text":"héllo"}""")
        assertTrue(len >= 0)
    }

    @Test fun multithreadedCalls() {
        val threads = 6
        val iters = 80
        val latch = CountDownLatch(threads)
        val errs = AtomicInteger(0)
        repeat(threads) { t ->
            Thread {
                try {
                    var acc = 0
                    repeat(iters) { i ->
                        acc += Ffi.roll(100L + t, 3, 6)
                        acc += Ffi.echoJsonLen("""{"i":$i}""")
                    }
                    assertTrue(acc > 0)
                } catch (e: Throwable) {
                    errs.incrementAndGet()
                } finally {
                    latch.countDown()
                }
            }.start()
        }
        latch.await()
        assertEquals(0, errs.get())
    }
}
