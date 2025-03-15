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
            (val && val.includes('youtube.com')) ||
            'Please enter a valid YouTube URL',
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
    <div v-if="isQuerying || foundSongs.length > 0" class="q-mt-xl">
      <!-- Status Messages -->
      <div v-if="isQuerying" class="text-subtitle2 text-bold text-primary">
        QUERY STATUS
        <q-spinner color="primary" size="1.5em" class="q-ml-sm" />
      </div>
      <div v-if="queryStatus" class="text-subtitle2 q-mb-md text-grey-6">
        {{ queryStatus }}
      </div>

      <!-- Found Songs List -->
      <div
        v-if="foundSongs.length > 0"
        class="text-subtitle2 text-bold text-primary q-mt-md"
      >
        FOUND SONGS
      </div>
      <div
        v-if="foundSongs.length > 0"
        class="row justify-center q-mt-sm"
        style="max-width: 725px; margin: auto"
      >
        <div class="col-10">
          <q-list bordered separator class="bg-dark rounded-borders">
            <q-item v-for="(song, index) in foundSongs" :key="index">
              <q-item-section avatar>
                <q-checkbox v-model="song.selected" />
              </q-item-section>
              <q-item-section>
                <q-item-label>{{ song.title }}</q-item-label>
                <q-item-label caption>{{ song.artist }}</q-item-label>
                <q-item-label caption v-if="song.source" class="text-primary">
                  Source: {{ song.source }}
                </q-item-label>
              </q-item-section>
            </q-item>
          </q-list>

          <div v-if="foundSongs.length > 2000" class="text-negative q-mt-md">
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
          <div v-if="foundSongs.length > 0">
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
            </div>

            <q-list
              bordered
              separator
              class="bg-dark rounded-borders"
              style="max-height: 60vh; overflow-y: auto"
            >
              <q-item v-for="(song, index) in foundSongs" :key="index">
                <q-item-section avatar>
                  <q-checkbox v-model="song.selected" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ song.title }}</q-item-label>
                  <q-item-label caption>{{ song.artist }}</q-item-label>
                  <q-item-label caption v-if="song.source" class="text-primary">
                    Source: {{ song.source }}
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>

            <div v-if="foundSongs.length > 2000" class="text-negative q-mt-md">
              Warning: Spotify has a query limit of 2,000 songs per 24 hours,
              which applies to both auto-tagging and audio features.
            </div>

            <div class="text-subtitle2 q-mt-md">
              {{ getSelectedSongsCount() }} of {{ foundSongs.length }} songs
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
  source?: string;
}

interface IPCResponse {
  action: string;
  songs?: FoundSong[];
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
const foundSongs = ref<FoundSong[]>([]);
const cliDialog = ref(false);
const isQuerying = ref(false);
const queryStatus = ref("");

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
  if (json.action === "analyzeSongs" && json.songs) {
    foundSongs.value = json.songs.map((song: any) => ({
      ...song,
      selected: true,
      source: determineSongSource(song),
    }));

    isQuerying.value = false;
    $q.loading.hide();

    if (foundSongs.value.length === 0) {
      queryStatus.value = "No songs found.";
      $q.notify({
        type: "warning",
        message: "No songs found with current settings.",
        position: "top",
      });
    } else {
      // Update status based on URL type
      if (url.value.includes("youtube.com")) {
        if (urlPreview.value?.contentType === "Channel") {
          queryStatus.value = `Found YouTube channel ${urlPreview.value.title}, ${foundSongs.value.length} songs extracted.`;
        } else if (urlPreview.value?.contentType === "Playlist") {
          queryStatus.value = `Found YouTube playlist ${urlPreview.value.title}, ${foundSongs.value.length} songs extracted.`;
        } else {
          queryStatus.value = `Found ${foundSongs.value.length} songs from YouTube.`;
        }
      } else if (url.value.includes("spotify.com")) {
        queryStatus.value = `Found ${foundSongs.value.length} songs from Spotify.`;
      } else if (url.value.includes("soundcloud.com")) {
        queryStatus.value = `Found ${foundSongs.value.length} songs from SoundCloud.`;
      } else {
        queryStatus.value = `Found ${foundSongs.value.length} songs.`;
      }

      $q.notify({
        type: "positive",
        message: `Found ${foundSongs.value.length} songs.`,
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
      const selectedSongs = foundSongs.value.filter((song) => song.selected);
      $q.notify({
        type: "positive",
        message: `${selectedSongs.length} songs downloaded successfully!`,
        position: "top",
      });

      // Reset the song list after successful download
      if (!confirmBeforeDownload.value) {
        foundSongs.value = [];
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
    foundSongs.value = [];
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

// Helper function to determine song source
function determineSongSource(song: FoundSong): string {
  if (song.video_url && song.video_url.includes("youtube.com")) {
    if (song.timestamp !== undefined) {
      return "Video description with timestamp";
    } else {
      return "Video title/description";
    }
  } else if (song.source) {
    return song.source;
  } else {
    return `Shazam (${Math.round(shazamConfidence.value * 100)}% confidence)`;
  }
}

// Helper function to select or deselect all songs
function selectAllSongs(selected: boolean): void {
  foundSongs.value.forEach((song) => {
    song.selected = selected;
  });
}

// Helper function to count selected songs
function getSelectedSongsCount(): number {
  return foundSongs.value.filter((song) => song.selected).length;
}

async function startDownload() {
  try {
    // If we already have songs from a previous query, use those
    if (foundSongs.value.length > 0) {
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
    foundSongs.value = [];
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
    // Filter only selected songs
    const selectedSongs = foundSongs.value.filter((song) => song.selected);

    if (selectedSongs.length === 0) {
      $q.notify({
        type: "warning",
        message: "No songs selected for download.",
        position: "top",
      });
      return;
    }

    $q.loading.show({
      message: "Downloading selected songs...",
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
</style>
