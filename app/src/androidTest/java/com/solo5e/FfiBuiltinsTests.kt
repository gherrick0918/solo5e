package com.solo5e

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class FfiBuiltinsTests {
    @Test
    fun duelMany_with_builtins() {
        val cfg = """
            {
              "target_id": "poison_goblin",
              "weapons_id": "basic",
              "weapon": "longsword",
              "seed": 1,
              "actor_hp": 12,
              "samples": 40
            }
        """.trimIndent()
        val raw = Ffi.simulateDuelManyJson(cfg)
        val root = JSONObject(raw)
        assertTrue(root.getBoolean("ok"))
        val r = root.getJSONObject("result")
        assertEquals(40, r.getInt("samples"))
        val sum = r.getInt("actor_wins") + r.getInt("enemy_wins") + r.getInt("draws")
        assertEquals(40, sum)
    }

    @Test
    fun encounter_with_builtins() {
        val cfg = """
            {
              "encounter_id": "goblin_ambush",
              "seed": 4242,
              "actor_hp": 10
            }
        """.trimIndent()
        val raw = Ffi.simulateEncounterJson(cfg)
        val root = JSONObject(raw)
        assertTrue(root.getBoolean("ok"))
        val r = root.getJSONObject("result")
        assertTrue(r.getInt("rounds") > 0)
    }
}
