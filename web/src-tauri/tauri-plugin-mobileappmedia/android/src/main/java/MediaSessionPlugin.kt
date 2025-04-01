package com.plugin.mobileappmedia

import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject

@InvokeArg
class MediaSessionArgs {
    lateinit var title: String
    lateinit var artist: String
    lateinit var artworkUrl: String
    var duration: Double = 0.0
}

@InvokeArg
class PlaybackStateArgs {
    var playState: Boolean = false
    var position: Double = 0.0
}

@InvokeArg
class TestBooleanArg {
    var value: Boolean = false
}

@TauriPlugin
class MediaSessionPlugin(private val activity: Activity): Plugin(activity) {
    private val example = Example() // This is needed for the ping command
    private val notificationManager: NotificationManager by lazy {
        activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    }
    private val CHANNEL_ID = "PinePods_Media_Channel"
    private val NOTIFICATION_ID = 1337

    // Track current state
    private var playState = false
    private var currentTitle = ""
    private var currentArtist = ""

    init {
        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val name = "PinePods Media"
            val descriptionText = "PinePods Media Playback"
            val importance = NotificationManager.IMPORTANCE_LOW
            val channel = NotificationChannel(CHANNEL_ID, name, importance).apply {
                description = descriptionText
            }
            notificationManager.createNotificationChannel(channel)
        }
    }

    @Command
    fun testBoolean(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(TestBooleanArg::class.java)
            System.out.println("PINEPODS_DEBUG: testBoolean called with value=${args.value}")

            val result = JSObject()
            invoke.resolve(result)
        } catch (e: Exception) {
            System.out.println("PINEPODS_DEBUG: Error in testBoolean: ${e.message}")
            invoke.reject("Failed: ${e.message}")
        }
    }

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)
        val ret = JSObject()
        ret.put("value", example.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    @Command
    fun registerMediaSession(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(MediaSessionArgs::class.java)

            // Save metadata
            currentTitle = args.title
            currentArtist = args.artist

            // Always start with paused state
            playState = false

            // Show notification
            showSimpleNotification(false)

            val result = JSObject()
            invoke.resolve(result)
        } catch (e: Exception) {
            invoke.reject("Failed to register media session: ${e.message}")
        }
    }

    @Command
    fun updatePlaybackState(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(PlaybackStateArgs::class.java)
            System.out.println("PINEPODS_DEBUG: updatePlaybackState called with playState=${args.playState}")
            // Always update state
            isPlaying = args.playState

            // Show updated notification
            showSimpleNotification(args.playState)

            val result = JSObject()
            invoke.resolve(result)
        } catch (e: Exception) {
            System.out.println("PINEPODS_DEBUG: Error in updatePlaybackState: ${e.message}")
            invoke.reject("Failed to update playback state: ${e.message}")
        }
    }

    private fun showSimpleNotification(playing: Boolean) {
        try {
            System.out.println("PINEPODS_DEBUG: showSimpleNotification called with playing=$playing")

            // Create intent to open app
            val intent = activity.packageManager.getLaunchIntentForPackage(activity.packageName)
            val pendingIntent = PendingIntent.getActivity(
                activity,
                0,
                intent,
                PendingIntent.FLAG_IMMUTABLE
            )

            // Build basic notification
            val builder = NotificationCompat.Builder(activity, CHANNEL_ID)
                .setContentTitle(currentTitle)
                .setContentText(currentArtist)
                .setSmallIcon(if (playing) android.R.drawable.ic_media_play else android.R.drawable.ic_media_pause)
                .setContentIntent(pendingIntent)
                .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                .setOngoing(playing)

            // Show notification
            notificationManager.notify(NOTIFICATION_ID, builder.build())

            System.out.println("PINEPODS_DEBUG: Notification shown with playing=$playing")
        } catch (e: Exception) {
            System.out.println("PINEPODS_DEBUG: Error showing notification: ${e.message}")
        }
    }
}
