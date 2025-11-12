package com.solo5e

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.json.JSONObject
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class FfiSimTests {
    @Test
    fun duelJsonSmoke() {
        val cfg = """
            {
              "target_path": "content/targets/poison_goblin.json",
              "weapon": "longsword",
              "weapons_path": "content/weapons/basic.json",
              "seed": 2025,
              "actor_conditions": [],
              "enemy_conditions": [],
              "actor_hp": 12
            }
        """.trimIndent()

        val raw = Ffi.simulateDuelJson(cfg)
        val root = JSONObject(raw)
        assertTrue(root.getBoolean("ok"))
        val res = root.getJSONObject("result")
        val rounds = res.getInt("rounds")
        val winner = res.getString("winner")
        assertTrue(rounds > 0)
        assertTrue(winner == "actor" || winner == "enemy" || winner == "draw")
    }
}
