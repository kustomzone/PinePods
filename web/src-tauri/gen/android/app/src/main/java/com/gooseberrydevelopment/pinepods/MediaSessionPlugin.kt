package com.gooseberrydevelopment.pinepods

import android.app.Activity
import android.graphics.BitmapFactory
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.support.v4.media.MediaMetadataCompat
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
    var isPlaying: Boolean = false
    var position: Double = 0.0
}

@TauriPlugin
class MediaSessionPlugin(activity: Activity): Plugin(activity) {
    private var mediaSession: MediaSessionCompat? = null
    private val mActivity = activity
    
    // Removed the load() method entirely
    
    @Command
    fun registerMediaSession(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(MediaSessionArgs::class.java)
            
            if (mediaSession == null) {
                mediaSession = MediaSessionCompat(mActivity, "PinePods")
                mediaSession?.isActive = true
            }
            
            val metadataBuilder = MediaMetadataCompat.Builder()
                .putString(MediaMetadataCompat.METADATA_KEY_TITLE, args.title)
                .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, args.artist)
                .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, (args.duration * 1000).toLong())
            
            mediaSession?.setMetadata(metadataBuilder.build())
            
            invoke.resolve(JSObject())
        } catch (e: Exception) {
            invoke.reject("Failed to register media session: ${e.message}")
        }
    }
    
    @Command
    fun updatePlaybackState(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(PlaybackStateArgs::class.java)
            
            if (mediaSession != null) {
                val stateBuilder = PlaybackStateCompat.Builder()
                
                val state = if (args.isPlaying) {
                    PlaybackStateCompat.STATE_PLAYING
                } else {
                    PlaybackStateCompat.STATE_PAUSED
                }
                
                stateBuilder.setState(
                    state,
                    (args.position * 1000).toLong(),
                    1.0f
                )
                
                val actions = PlaybackStateCompat.ACTION_PLAY or
                            PlaybackStateCompat.ACTION_PAUSE or
                            PlaybackStateCompat.ACTION_PLAY_PAUSE or
                            PlaybackStateCompat.ACTION_SEEK_TO
                
                stateBuilder.setActions(actions)
                
                mediaSession?.setPlaybackState(stateBuilder.build())
            }
            
            invoke.resolve(JSObject())
        } catch (e: Exception) {
            invoke.reject("Failed to update playback state: ${e.message}")
        }
    }
}