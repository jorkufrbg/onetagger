<template>
  <q-page class="songs-downloader q-pa-md">
    <!-- URL Input -->
    <div class="text-subtitle2 text-bold text-primary">ENTER URL</div>
    <div class="text-subtitle2 q-mb-md text-grey-6">
      Provide a URL for a YouTube channel, playlist, or video, or for Spotify,
      YouTube, or SoundCloud.
    </div>
    <div
      class="row justify-center input"
      style="max-width: 725px; margin: auto"
    >
      <div class="col-1"></div>
      <q-input
        filled
        class="col-10"
        label="URL"
        v-model="url"
        :rules="[
          (val) =>
            (val && (val.includes('youtube.com') || val.includes('youtu.be') || 
                    val.includes('spotify.com') || val.includes('soundcloud.com'))) ||
            'Please enter a valid YouTube, Spotify, or SoundCloud URL',
        ]"
      >
      </q-input>
      <div class="col-1">
        <q-icon
          name="mdi-help-circle-outline text-grey-6"
          class="path-tooltip q-mx-sm q-pt-md q-mt-xs"
        >
          <q-tooltip
            >Submit a YouTube, Spotify, or SoundCloud URL to analyze the songs
            within.</q-tooltip
          >
        </q-icon>
      </div>
    </div>

    <!-- Shazam Confidence Score -->
    <div class="text-subtitle2 text-bold text-primary q-mt-xl">
      SHAZAM CONFIDENCE
    </div>
    <div class="text-subtitle2 q-mb-md text-grey-6">
      Adjust the confidence threshold for song detection
    </div>
    <div class="row justify-center" style="max-width: 550px; margin: auto">
      <q-slider
        v-model="shazamConfidence"
        :min="0.0"
        :max="1.0"
        :step="0.05"
        label
        label-always
        class="slider q-mt-xl q-my-sm q-pb-lg col-10"
        label-text-color="black"
        :label-value="'Confidence: ' + Math.round(shazamConfidence * 100) + '%'"
      >
      </q-slider>
      <q-icon
        name="mdi-help-circle-outline text-grey-6"
        class="q-pt-md q-mx-sm slider-tooltip"
      >
        <q-tooltip>
          Higher values mean more accurate but fewer matches. This will
          prioritise song descriptions from the URL over Shazam results. Lower
          values mean more matches but less accurate. A value of 0.75 is
          recommended.
        </q-tooltip>
      </q-icon>
    </div>

    <!-- Query URL Button -->
    <div
      class="row justify-center q-mt-md"
      style="max-width: 550px; margin: auto"
    >
      <q-btn
        color="secondary"
        label="Query URL"
        :disable="!url"
        @click="queryUrl"
        push
        class="q-mb-lg"
      />
      <q-icon
        name="mdi-help-circle-outline text-grey-6"
        class="q-pt-md q-mx-sm"
      >
        <q-tooltip>
          Query the URL to find songs before downloading. This will analyze the
          content and show you what songs were found.
        </q-tooltip>
      </q-icon>
    </div>

    <!-- Output Directory -->
    <div class="text-subtitle2 text-bold text-primary q-mt-lg">
      SELECT OUTPUT DIRECTORY
    </div>
    <div class="text-subtitle2 q-mb-md text-grey-6">
      Select where to save downloaded songs, copy/paste path directly or
      <span class="q-px-sm text-caption text-bold click-highlight">CLICK</span>
      the
      <q-icon name="mdi-open-in-app"></q-icon>
      icon to browse
    </div>
    <div
      class="row justify-center input"
      style="max-width: 725px; margin: auto"
    >
      <div class="col-1"></div>
      <q-input filled class="col-10" label="Path" v-model="config.path">
        <template v-slot:append>
          <q-btn
            round
            dense
            flat
            icon="mdi-open-in-app"
            class="text-grey-4"
            @click="browse"
          ></q-btn>
        </template>
      </q-input>

      <div class="col-1">
        <q-icon
          name="mdi-help-circle-outline text-grey-6"
          class="path-tooltip q-mx-sm q-pt-md q-mt-xs"
        >
          <q-tooltip>Choose where to save the downloaded songs</q-tooltip>
        </q-icon>
      </div>
    </div>

    <!-- Confirm Songs Switch -->
    <div class="text-subtitle2 text-bold text-primary q-mt-xl">
      CONFIRMATION
    </div>
    <div class="row justify-center" style="max-width: 550px; margin: auto">
      <q-toggle
        v-model="confirmBeforeDownload"
        label="Confirm Songs Before Download"
        class="q-mt-md"
      />
      <q-icon
        name="mdi-help-circle-outline text-grey-6"
        class="q-pt-md q-mx-sm"
      >
        <q-tooltip>
          Highly recommended to keep enabled as song extraction may not always
          be accurate, especially for YouTube and SoundCloud. For Spotify, while
          more reliable, it's still good practice to verify the correct URL was
          provided.
        </q-tooltip>
      </q-icon>
    </div>

    <!-- URL Preview -->
    <div
      v-if="urlPreview"
      class="row justify-center q-mt-sm"
      style="max-width: 725px; margin: auto"
    >
      <div
        class="col-10 text-grey-6"
        style="
          font-size: 0.9em;
          text-align: left;
          padding: 8px 12px;
          background: rgba(255, 255, 255, 0.05);
          border-radius: 4px;
        "
      >
        <template v-if="urlPreview.type === 'youtube'">
          <span class="text-weight-medium"
            >YouTube {{ urlPreview.contentType }}:</span
          >
          {{ urlPreview.title }}
          <div v-if="urlPreview.description">{{ urlPreview.description }}</div>
        </template>
        <template v-else-if="urlPreview.type === 'spotify'">
          <span class="text-weight-medium"
            >Spotify {{ urlPreview.contentType }}:</span
          >
          {{ urlPreview.title }}
          <div v-if="urlPreview.description">{{ urlPreview.description }}</div>
        </template>
        <template v-else-if="urlPreview.type === 'soundcloud'">
          <span class="text-weight-medium"
            >SoundCloud {{ urlPreview.contentType }}:</span
          >
          {{ urlPreview.title }}
          <div v-if="urlPreview.description">{{ urlPreview.description }}</div>
        </template>
      </div>
    </div>

    <!-- Query Status and Results -->
    <div v-if="isQuerying || foundVideos.length > 0" class="q-mt-xl">
      <!-- Status Messages -->
      <div v-if="isQuerying" class="text-subtitle2 text-bold text-primary">
        QUERY STATUS
        <q-spinner color="primary" size="1.5em" class="q-ml-sm" />
      </div>
      <div v-if="queryStatus" class="text-subtitle2 q-mb-md text-grey-6">
        {{ queryStatus }}
      </div>

      <!-- Found Videos with Songs List -->
      <div
        v-if="foundVideos.length > 0"
        class="text-subtitle2 text-bold text-primary q-mt-md"
      >
        FOUND VIDEOS/PLAYLISTS
      </div>
      <div
        v-if="foundVideos.length > 0"
        class="row justify-center q-mt-sm"
        style="max-width: 800px; margin: auto"
      >
        <div class="col-12">
          <!-- Bulk Actions Bar -->
          <div class="row q-mb-md justify-center">
            <div class="col-10 row justify-between items-center bg-dark rounded-borders q-pa-sm">
              <div>
                <q-btn
                  flat
                  dense
                  color="primary"
                  label="Select All"
                  @click="selectAllSongs(true)"
                  class="q-mx-xs"
                  size="sm"
                />
                <q-btn
                  flat
                  dense
                  color="primary"
                  label="Deselect All"
                  @click="selectAllSongs(false)"
                  class="q-mx-xs"
                  size="sm"
                />
                <q-btn
                  flat
                  dense
                  color="primary"
                  label="Expand All"
                  @click="expandAllVideos(true)"
                  class="q-mx-xs"
                  size="sm"
                />
                <q-btn
                  flat
                  dense
                  color="primary"
                  label="Collapse All"
                  @click="expandAllVideos(false)"
                  class="q-mx-xs"
                  size="sm"
                />
              </div>
              <div class="text-caption">
                {{ getSelectedSongsCount() }} of {{ totalSongsCount }} songs selected
              </div>
            </div>
          </div>

          <!-- Videos & Songs List -->
          <q-list bordered separator class="bg-dark rounded-borders">
            <div v-for="(video, videoIndex) in foundVideos" :key="videoIndex">
              <!-- Video Header - Expandable -->
              <q-expansion-item
                v-model="video.expanded"
                :icon="getPlatformIcon(video.source_url)"
                :label="video.title"
                header-class="video-header"
                expand-icon-class="text-primary"
              >
                <template v-slot:header>
                  <q-item-section avatar>
                    <q-checkbox 
                      v-model="video.allSelected" 
                      @update:model-value="toggleAllSongsInVideo(videoIndex)"
                    />
                  </q-item-section>
                  
                  <q-item-section>
                    <q-item-label class="ellipsis-2-lines">{{ video.title }}</q-item-label>
                    <q-item-label caption class="text-grey-5">
                      {{ video.songs.length }} songs · {{ getPlatformName(video.source_url) }}
                    </q-item-label>
                  </q-item-section>
                  
                  <q-item-section side>
                    <q-btn
                      flat
                      round
                      dense
                      color="primary"
                      icon="mdi-link"
                      @click.stop="openSourceUrl(video.source_url)"
                    >
                      <q-tooltip>Open source URL</q-tooltip>
                    </q-btn>
                  </q-item-section>
                </template>

                <!-- Song List for this Video -->
                <q-list separator>
                  <q-item v-for="(song, songIndex) in video.songs" :key="songIndex">
                    <q-item-section avatar>
                      <q-checkbox 
                        v-model="song.selected" 
                        @update:model-value="updateVideoSelectStatus(videoIndex)" 
                      />
                    </q-item-section>
                    
                    <q-item-section>
                      <div class="row no-wrap">
                        <div class="col-12 col-sm-6 q-pr-md">
                          <q-input 
                            dense 
                            v-model="song.title" 
                            label="Title" 
                            class="q-mb-xs"
                            stack-label
                          />
                        </div>
                        <div class="col-12 col-sm-6">
                          <q-input 
                            dense 
                            v-model="song.artist" 
                            label="Artist" 
                            class="q-mb-xs"
                            stack-label
                          />
                        </div>
                      </div>
                      
                      <div class="row items-center q-mt-xs">
                        <q-chip 
                          v-if="song.timestamp" 
                          size="sm" 
                          dense 
                          class="text-caption"
                          color="secondary"
                          text-color="white"
                        >
                          {{ formatTimestamp(song.timestamp) }}
                        </q-chip>
                        
                        <q-chip 
                          v-if="song.match_confidence" 
                          size="sm" 
                          dense 
                          class="text-caption q-ml-xs"
                          :color="getConfidenceColor(song.match_confidence)"
                          text-color="white"
                        >
                          {{ Math.round(song.match_confidence * 100) }}% match
                        </q-chip>
                        
                        <div class="q-ml-auto">
                          <q-btn 
                            flat 
                            dense 
                            round 
                            color="grey-6" 
                            icon="mdi-undo" 
                            size="sm"
                            @click="resetSong(song, videoIndex, songIndex)"
                          >
                            <q-tooltip>Reset to original</q-tooltip>
                          </q-btn>
                        </div>
                      </div>
                    </q-item-section>
                  </q-item>
                </q-list>
              </q-expansion-item>
            </div>
          </q-list>

          <div v-if="totalSongsCount > 2000" class="text-negative q-mt-md text-center">
            Warning: Spotify has a query limit of 2,000 songs per 24 hours,
            which applies to both auto-tagging and audio features.
          </div>
        </div>
      </div>
    </div>

    <!-- Auto Tag Option -->
    <div class="text-subtitle2 text-bold text-primary q-mt-xl">
      AUTO TAG OPTIONS
    </div>
    <div class="row justify-center" style="max-width: 550px; margin: auto">
      <q-toggle
        v-model="enableAutoTag"
        label="Enable Auto Tagging"
        class="q-mt-md"
      />
      <q-icon
        name="mdi-help-circle-outline text-grey-6"
        class="q-pt-md q-mx-sm"
      >
        <q-tooltip> Enable to automatically tag downloaded songs </q-tooltip>
      </q-icon>
    </div>

    <!-- Auto Tag Config -->
    <div
      v-if="enableAutoTag"
      class="row justify-center q-mt-md"
      style="max-width: 725px; margin: auto"
    >
      <q-input
        v-model="autoTagConfig"
        filled
        type="textarea"
        label="Auto Tag Configuration"
        class="col-10"
        :rules="[
          (val) =>
            (val && val.length > 0) || 'Auto tag configuration is required',
        ]"
      />
    </div>

    <!-- Audio Features Toggle -->
    <div class="text-subtitle2 text-bold text-primary q-mt-xl">
      AUDIO FEATURES
    </div>
    <div class="row justify-center" style="max-width: 550px; margin: auto">
      <q-toggle
        v-model="enableAudioFeatures"
        label="Enable Audio Features"
        class="q-mt-md"
      />
      <q-icon
        name="mdi-help-circle-outline text-grey-6"
        class="q-pt-md q-mx-sm"
      >
        <q-tooltip>
          Enable to analyze audio features of downloaded songs
        </q-tooltip>
      </q-icon>
    </div>

    <!-- Submit Button -->
    <div class="q-mt-xl q-mb-xl text-center">
      <div class="row">
        <!-- CLI FAB -->
        <div class="q-mr-md q-mt-md col-12 text-right">
          <q-btn
            class="bg-grey-9"
            flat
            round
            icon="mdi-console"
            color="grey-4"
            @click="cliDialog = true"
          >
            <q-tooltip
              anchor="top middle"
              self="bottom middle"
              :offset="[10, 10]"
            >
              <span class="text-weight-medium">CLI Version Config</span>
            </q-tooltip>
          </q-btn>
        </div>

        <div class="col-12 text-center">
          <q-btn
            color="primary"
            label="Start Download"
            :disable="!isValid"
            @click="startDownload"
            push
            size="lg"
          />
        </div>
      </div>
    </div>

    <!-- Confirmation Dialog -->
    <q-dialog v-model="showConfirmation">
      <q-card style="min-width: 500px; max-width: 90vw; max-height: 90vh">
        <q-card-section>
          <div class="text-h6">Confirm Songs</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <div v-if="foundVideos.length > 0">
            <div class="text-subtitle1 q-mb-sm">
              Select songs to download:
              <q-btn
                flat
                dense
                color="primary"
                label="Select All"
                @click="selectAllSongs(true)"
                class="q-ml-sm"
              />
              <q-btn
                flat
                dense
                color="primary"
                label="Deselect All"
                @click="selectAllSongs(false)"
                class="q-ml-sm"
              />
              <q-btn
                flat
                dense
                color="primary"
                label="Expand All"
                @click="expandAllVideos(true)"
                class="q-ml-sm"
              />
              <q-btn
                flat
                dense
                color="primary"
                label="Collapse All"
                @click="expandAllVideos(false)"
                class="q-ml-sm"
              />
            </div>

            <q-list
              bordered
              separator
              class="bg-dark rounded-borders"
              style="max-height: 60vh; overflow-y: auto"
            >
              <div v-for="(video, videoIndex) in foundVideos" :key="videoIndex">
                <q-expansion-item
                  v-model="video.expanded"
                  :icon="getPlatformIcon(video.source_url)"
                  :label="video.title"
                  header-class="video-header"
                  expand-icon-class="text-primary"
                >
                  <template v-slot:header>
                    <q-item-section avatar>
                      <q-checkbox 
                        v-model="video.allSelected" 
                        @update:model-value="toggleAllSongsInVideo(videoIndex)"
                      />
                    </q-item-section>
                    
                    <q-item-section>
                      <q-item-label class="ellipsis-2-lines">{{ video.title }}</q-item-label>
                      <q-item-label caption class="text-grey-5">
                        {{ video.songs.length }} songs · {{ getPlatformName(video.source_url) }}
                      </q-item-label>
                    </q-item-section>
                  </template>

                  <q-list separator>
                    <q-item v-for="(song, songIndex) in video.songs" :key="songIndex">
                      <q-item-section avatar>
                        <q-checkbox 
                          v-model="song.selected" 
                          @update:model-value="updateVideoSelectStatus(videoIndex)" 
                        />
                      </q-item-section>
                      
                      <q-item-section>
                        <q-item-label>{{ song.title }}</q-item-label>
                        <q-item-label caption>{{ song.artist }}</q-item-label>
                        
                        <div class="row items-center q-mt-xs">
                          <q-chip 
                            v-if="song.timestamp" 
                            size="sm" 
                            dense 
                            class="text-caption"
                            color="secondary"
                            text-color="white"
                          >
                            {{ formatTimestamp(song.timestamp) }}
                          </q-chip>
                          
                          <q-chip 
                            v-if="song.match_confidence" 
                            size="sm" 
                            dense 
                            class="text-caption q-ml-xs"
                            :color="getConfidenceColor(song.match_confidence)"
                            text-color="white"
                          >
                            {{ Math.round(song.match_confidence * 100) }}% match
                          </q-chip>
                        </div>
                      </q-item-section>
                    </q-item>
                  </q-list>
                </q-expansion-item>
              </div>
            </q-list>

            <div v-if="totalSongsCount > 2000" class="text-negative q-mt-md">
              Warning: Spotify has a query limit of 2,000 songs per 24 hours,
              which applies to both auto-tagging and audio features.
            </div>

            <div class="text-subtitle2 q-mt-md">
              {{ getSelectedSongsCount() }} of {{ totalSongsCount }} songs
              selected
            </div>
          </div>
          <div v-else>No songs found with current settings</div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat label="Cancel" color="primary" v-close-popup />
          <q-btn
            flat
            label="Download Selected"
            color="primary"
            @click="confirmDownload"
            v-close-popup
            :disable="getSelectedSongsCount() === 0"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- CLI Dialog -->
    <q-dialog v-model="cliDialog">
      <CliDialog
        :config="cliConfig"
        command="songdownloader"
        extra="--confidence 0.75"
      ></CliDialog>
    </q-dialog>
  </q-page>
</template>

<script lang="ts" setup>
import { ref, computed, watch } from "vue";
import { get1t } from "../scripts/onetagger";
import { useQuasar } from "quasar";
import CliDialog from "../components/CliDialog.vue";

interface FoundSong {
  title: string;
  artist: string;
  video_url: string;
  timestamp?: number;
  selected?: boolean;
  match_confidence?: number;
  downloaded?: boolean;
  // Store original values for reset
  original_title?: string;
  original_artist?: string;
}

interface VideoInfo {
  title: string;
  source_url: string;
  songs: FoundSong[];
  expanded: boolean;
  allSelected: boolean;
}

interface IPCResponse {
  action: string;
  videos?: VideoInfo[];
  success?: boolean;
  error?: string;
}

interface URLPreview {
  type: "youtube" | "spotify" | "soundcloud";
  contentType: string; // 'Channel', 'Playlist', 'Video', 'Artist', 'Album', etc.
  title: string;
  description?: string;
}

const $1t = get1t();
const $q = useQuasar();

const config = ref({
  path: "",
});

const url = ref("");
const urlPreview = ref<URLPreview | null>(null);
const shazamConfidence = ref(0.75);
const confirmBeforeDownload = ref(true);
const enableAutoTag = ref(false);
const autoTagConfig = ref("");
const enableAudioFeatures = ref(false);
const showConfirmation = ref(false);
const foundVideos = ref<VideoInfo[]>([]);
const cliDialog = ref(false);
const isQuerying = ref(false);
const queryStatus = ref("");
const totalSongsCount = computed(() => {
  return foundVideos.value.reduce((total, video) => total + video.songs.length, 0);
});

// Watch URL changes
watch(
  () => url.value,
  async (newUrlValue) => {
    if (!newUrlValue) {
      urlPreview.value = null;
      return;
    }

    // Basic URL validation
    const isYoutube =
      newUrlValue.includes("youtube.com") || newUrlValue.includes("youtu.be");
    const isSpotify = newUrlValue.includes("spotify.com");
    const isSoundcloud = newUrlValue.includes("soundcloud.com");

    if (!isYoutube && !isSpotify && !isSoundcloud) {
      urlPreview.value = null;
      return;
    }

    try {
      // Show loading state
      $q.loading.show({
        message: "Fetching URL information...",
      });

      // Type assertion for the result
      const apiResult = await $1t.send("songdownloader_getUrlInfo", {
        url: newUrlValue,
      });

      const result = apiResult as unknown as {
        success?: boolean;
        platform?: string;
        contentType?: string;
        title?: string;
        description?: string;
        error?: string;
      };

      $q.loading.hide();

      if (result && result.success) {
        urlPreview.value = {
          type: result.platform as "youtube" | "spotify" | "soundcloud",
          contentType: result.contentType || "",
          title: result.title || "",
          description: result.description,
        };
      } else {
        urlPreview.value = null;
        if (result && result.error) {
          $q.notify({
            type: "warning",
            message: result.error,
            position: "top",
          });
        }
      }
    } catch (error) {
      $q.loading.hide();
      console.error("Error fetching URL preview:", error);
      urlPreview.value = null;
      $q.notify({
        type: "negative",
        message: "Failed to fetch URL information",
        position: "top",
      });
    }
  },
  { flush: "post" }
); // Use throttle instead of debounce

const isValid = computed(() => {
  if (!url.value || !config.value.path) return false;
  if (enableAutoTag.value && !autoTagConfig.value) return false;
  // Update validation to include other platforms
  const isValidUrl =
    url.value.includes("youtube.com") ||
    url.value.includes("spotify.com") ||
    url.value.includes("soundcloud.com");
  return isValidUrl;
});

// Register the song downloader event handler
$1t.onSongDownloaderEvent = (json: any) => {
  if (json.context === "sd" && json.path) {
    config.value.path = json.path;
  }

  // Handle analyzeSongs response
  if (json.action === "analyzeSongs" && json.videos) {
    // Convert backend videos data to our UI structure
    foundVideos.value = json.videos.map((video: any) => {
      const songsWithMeta = video.songs.map((song: any) => ({
        ...song,
        selected: true,
        // Save original values for reset functionality
        original_title: song.title,
        original_artist: song.artist
      }));
      
      return {
        title: video.title,
        source_url: video.source_url || video.video_url,
        songs: songsWithMeta,
        expanded: true, // Start expanded
        allSelected: true // Start with all selected
      };
    });

    isQuerying.value = false;
    $q.loading.hide();

    const songCount = totalSongsCount.value;

    if (songCount === 0) {
      queryStatus.value = "No songs found.";
      $q.notify({
        type: "warning",
        message: "No songs found with current settings.",
        position: "top",
      });
    } else {
      // Update status based on URL type and content
      if (url.value.includes("youtube.com")) {
        if (urlPreview.value?.contentType === "channel") {
          queryStatus.value = `Found YouTube channel ${urlPreview.value.title} with ${foundVideos.value.length} videos, ${songCount} songs extracted.`;
        } else if (urlPreview.value?.contentType === "playlist") {
          queryStatus.value = `Found YouTube playlist ${urlPreview.value.title} with ${foundVideos.value.length} videos, ${songCount} songs extracted.`;
        } else {
          queryStatus.value = `Found ${songCount} songs from ${foundVideos.value.length} YouTube videos.`;
        }
      } else if (url.value.includes("spotify.com")) {
        if (urlPreview.value?.contentType === "playlist") {
          queryStatus.value = `Found Spotify playlist ${urlPreview.value.title} with ${songCount} songs.`;
        } else if (urlPreview.value?.contentType === "album") {
          queryStatus.value = `Found Spotify album ${urlPreview.value.title} with ${songCount} songs.`;
        } else if (urlPreview.value?.contentType === "artist") {
          queryStatus.value = `Found Spotify artist ${urlPreview.value.title} with ${songCount} songs.`;
        } else {
          queryStatus.value = `Found ${songCount} songs from Spotify.`;
        }
      } else if (url.value.includes("soundcloud.com")) {
        if (urlPreview.value?.contentType === "playlist") {
          queryStatus.value = `Found SoundCloud playlist with ${songCount} songs.`;
        } else {
          queryStatus.value = `Found ${songCount} songs from SoundCloud.`;
        }
      } else {
        queryStatus.value = `Found ${songCount} songs from ${foundVideos.value.length} sources.`;
      }

      $q.notify({
        type: "positive",
        message: `Found ${songCount} songs from ${foundVideos.value.length} sources.`,
        position: "top",
      });

      // If this was triggered from startDownload and confirmation is enabled, show the dialog
      if (confirmBeforeDownload.value) {
        showConfirmation.value = true;
      } else {
        // If confirmation is disabled, proceed directly to download
        confirmDownload();
      }
    }
  }

  // Handle error response
  if (json.action === "analyzeSongs" && json.error) {
    isQuerying.value = false;
    $q.loading.hide();
    queryStatus.value = "Error analyzing songs.";
    $q.notify({
      type: "negative",
      message: `Failed to analyze songs: ${json.error}`,
      position: "top",
    });
  }

  // Handle downloadSongs response
  if (json.action === "downloadSongs") {
    $q.loading.hide();

    if (json.success) {
      const selectedCount = getSelectedSongsCount();
      $q.notify({
        type: "positive",
        message: `${selectedCount} songs downloaded successfully!`,
        position: "top",
      });

      // Reset the song list after successful download
      if (!confirmBeforeDownload.value) {
        foundVideos.value = [];
        queryStatus.value = "";
      }
    } else {
      $q.notify({
        type: "negative",
        message: `Failed to download songs: ${json.error || "Unknown error"}`,
        position: "top",
      });
    }
  }
};

// CLI config for showing in dialog
const cliConfig = computed(() => {
  return {
    url: url.value,
    output: config.value.path,
    confidence: shazamConfidence.value,
    enableAutoTag: enableAutoTag.value,
    autoTagConfig: enableAutoTag.value ? autoTagConfig.value : undefined,
    enableAudioFeatures: enableAudioFeatures.value,
  };
});

function browse() {
  $1t.browse("songs", config.value.path);
}

async function queryUrl() {
  try {
    // Debug message
    console.log("queryUrl function called with URL:", url.value);

    // Reset previous results
    foundVideos.value = [];
    isQuerying.value = true;
    queryStatus.value = "Analyzing URL...";

    // Show loading state with status message
    $q.loading.show({
      message: "Analyzing URL and finding songs...",
    });

    // Debug message
    console.log("Sending analyzeSongs request with:", {
      url: url.value,
      confidence: shazamConfidence.value,
    });

    // Call the backend to analyze songs
    // The response will be handled by the onSongDownloaderEvent handler
    await $1t.send("analyzeSongs", {
      url: url.value,
      confidence: shazamConfidence.value,
    });

    // Note: We don't hide the loading indicator or set isQuerying to false here
    // because that will be handled by the onSongDownloaderEvent handler
  } catch (error) {
    console.error("Error analyzing songs:", error);
    queryStatus.value = "Error analyzing songs.";
    isQuerying.value = false;
    $q.loading.hide();
    $q.notify({
      type: "negative",
      message: "Failed to analyze songs. Please try again.",
      position: "top",
    });
  }
}

// Helper function to determine platform name
function getPlatformName(url: string): string {
  if (url.includes('youtube.com') || url.includes('youtu.be')) {
    return 'YouTube';
  } else if (url.includes('spotify.com')) {
    return 'Spotify';
  } else if (url.includes('soundcloud.com')) {
    return 'SoundCloud';
  }
  return 'Unknown';
}

// Helper function to get platform icon
function getPlatformIcon(url: string): string {
  if (url.includes('youtube.com') || url.includes('youtu.be')) {
    return 'mdi-youtube';
  } else if (url.includes('spotify.com')) {
    return 'mdi-spotify';
  } else if (url.includes('soundcloud.com')) {
    return 'mdi-soundcloud';
  }
  return 'mdi-music';
}

// Format a timestamp in seconds to mm:ss format
function formatTimestamp(timestamp: number | undefined): string {
  if (!timestamp) return '';
  
  const minutes = Math.floor(timestamp / 60);
  const seconds = Math.floor(timestamp % 60);
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

// Get confidence color based on match value
function getConfidenceColor(confidence: number | undefined): string {
  if (!confidence) return 'grey';
  
  if (confidence >= 0.9) return 'positive';
  if (confidence >= 0.7) return 'secondary';
  if (confidence >= 0.5) return 'warning';
  return 'negative';
}

// Helper function to select or deselect all songs across all videos
function selectAllSongs(selected: boolean): void {
  foundVideos.value.forEach((video) => {
    video.allSelected = selected;
    video.songs.forEach((song) => {
      song.selected = selected;
    });
  });
}

// Helper function to toggle expand/collapse all videos
function expandAllVideos(expanded: boolean): void {
  foundVideos.value.forEach((video) => {
    video.expanded = expanded;
  });
}

// Toggle all songs in a specific video
function toggleAllSongsInVideo(videoIndex: number): void {
  const video = foundVideos.value[videoIndex];
  video.songs.forEach((song) => {
    song.selected = video.allSelected;
  });
}

// Update video selection status based on its songs
function updateVideoSelectStatus(videoIndex: number): void {
  const video = foundVideos.value[videoIndex];
  const allSelected = video.songs.every((song) => song.selected);
  const anySelected = video.songs.some((song) => song.selected);
  
  video.allSelected = allSelected;
  // If indeterminate state is needed, could be implemented here
}

// Reset a song to its original values
function resetSong(song: FoundSong, videoIndex: number, songIndex: number): void {
  if (song.original_title) {
    song.title = song.original_title;
  }
  
  if (song.original_artist) {
    song.artist = song.original_artist;
  }
}

// Open the source URL in a new tab
function openSourceUrl(url: string): void {
  window.open(url, '_blank');
}

// Helper function to count selected songs
function getSelectedSongsCount(): number {
  return foundVideos.value.reduce((count, video) => {
    return count + video.songs.filter(song => song.selected).length;
  }, 0);
}

async function startDownload() {
  try {
    // If we already have songs from a previous query, use those
    if (foundVideos.value.length > 0) {
      if (confirmBeforeDownload.value) {
        showConfirmation.value = true;
      } else {
        // If confirmation is disabled, proceed directly to download
        await confirmDownload();
      }
      return;
    }

    // Otherwise, query for songs first
    // Reset previous results
    foundVideos.value = [];
    isQuerying.value = true;
    queryStatus.value = "Analyzing URL...";

    // Show loading state with status message
    $q.loading.show({
      message: "Analyzing URL and finding songs...",
    });

    // Call the backend to analyze songs
    // The response will be handled by the onSongDownloaderEvent handler
    await $1t.send("analyzeSongs", {
      url: url.value,
      confidence: shazamConfidence.value,
    });

    // Note: We don't hide the loading indicator or process the response here
    // because that will be handled by the onSongDownloaderEvent handler
    // The handler will also set up the confirmation dialog if needed
  } catch (error) {
    console.error("Error analyzing songs:", error);
    isQuerying.value = false;
    $q.loading.hide();
    $q.notify({
      type: "negative",
      message: "Failed to analyze songs. Please try again.",
      position: "top",
    });
  }
}

async function confirmDownload() {
  try {
    // Collect all selected songs from all videos
    const selectedSongs = [];
    
    for (const video of foundVideos.value) {
      for (const song of video.songs) {
        if (song.selected) {
          selectedSongs.push({
            title: song.title,
            artist: song.artist,
            video_url: video.source_url,
            timestamp: song.timestamp,
            match_confidence: song.match_confidence,
            // Add any other fields needed by the backend
          });
        }
      }
    }

    if (selectedSongs.length === 0) {
      $q.notify({
        type: "warning",
        message: "No songs selected for download.",
        position: "top",
      });
      return;
    }

    $q.loading.show({
      message: `Downloading ${selectedSongs.length} selected songs...`,
    });

    // Call the backend to download songs
    // The response will be handled by the onSongDownloaderEvent handler
    await $1t.send("downloadSongs", {
      url: url.value,
      outputPath: config.value.path,
      confidence: shazamConfidence.value,
      enableAutoTag: enableAutoTag.value,
      autoTagConfig: enableAutoTag.value ? autoTagConfig.value : null,
      enableAudioFeatures: enableAudioFeatures.value,
      songs: selectedSongs,
    });

    // Note: We don't hide the loading indicator or process the response here
    // because that will be handled by the onSongDownloaderEvent handler
  } catch (error) {
    console.error("Error downloading songs:", error);
    $q.loading.hide();
    $q.notify({
      type: "negative",
      message: "Failed to download songs. Please try again.",
      position: "top",
    });
  }
}
</script>

<style lang="scss" scoped>
.songs-downloader {
  min-height: calc(100vh - 50px);
  background: var(--q-dark);
  text-align: center;
  overflow-y: auto;
  display: block;
}

.input {
  margin-top: 8px;
  padding-left: 16px;
  padding-right: 16px;
}

.slider {
  max-width: 550px !important;
}

:deep(.q-slider__track-container) {
  background: rgba(255, 255, 255, 0.1);
}

:deep(.q-slider__selection) {
  background: var(--q-primary);
}

:deep(.q-slider__thumb) {
  color: var(--q-primary);
}

:deep(.q-field--filled .q-field__control) {
  background: rgba(255, 255, 255, 0.05);
}

:deep(.q-field--filled .q-field__control:hover) {
  background: rgba(255, 255, 255, 0.1);
}

.video-header {
  transition: background-color 0.3s;
}

.video-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

.ellipsis-2-lines {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
}

:deep(.q-expansion-item__content) {
  background: rgba(255, 255, 255, 0.02);
}
</style>
