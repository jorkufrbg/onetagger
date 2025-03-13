<template>
    <q-page class="songs-downloader q-pa-md">
        <!-- URL Input -->
        <div class='text-subtitle2 text-bold text-primary'>ENTER URL</div>
        <div class='text-subtitle2 q-mb-md text-grey-6'>Enter a YouTube channel, playlist, or video URL</div>
        <div class='row justify-center input' style='max-width: 725px; margin: auto;'>
            <div class='col-1'></div>
            <q-input 
                filled 
                class='col-10' 
                label='URL' 
                v-model='url'
                :rules="[val => val && val.includes('youtube.com') || 'Please enter a valid YouTube URL']"
            >
            </q-input>
            <div class='col-1'>
                <q-icon name='mdi-help-circle-outline text-grey-6' class='path-tooltip q-mx-sm q-pt-md q-mt-xs'>
                    <q-tooltip>Enter a YouTube URL to analyze for songs</q-tooltip>
                </q-icon>
            </div>
        </div>
    
        <!-- Output Directory -->        
        <div class='text-subtitle2 text-bold text-primary q-mt-lg'>SELECT OUTPUT DIRECTORY</div>
        <div class='text-subtitle2 q-mb-md text-grey-6'>
            Select where to save downloaded songs, copy/paste path directly or
            <span class='q-px-sm text-caption text-bold click-highlight'>CLICK</span> the 
            <q-icon name='mdi-open-in-app'></q-icon> 
            icon to browse
        </div>
        <div class='row justify-center input' style='max-width: 725px; margin: auto;'>
            <div class='col-1'></div>
            <q-input filled class='col-10' label='Path' v-model='config.path'>
                <template v-slot:append>
                    <q-btn round dense flat icon='mdi-open-in-app' class='text-grey-4' @click='browse'></q-btn>
                </template>
            </q-input>

            <div class='col-1'>
                <q-icon name='mdi-help-circle-outline text-grey-6' class='path-tooltip q-mx-sm q-pt-md q-mt-xs'>
                    <q-tooltip>Choose where to save the downloaded songs</q-tooltip>
                </q-icon>
            </div>
        </div>
    
        <!-- Shazam Confidence Score -->
        <div class='text-subtitle2 text-bold text-primary q-mt-xl'>SHAZAM CONFIDENCE</div>
        <div class='text-subtitle2 q-mb-md text-grey-6'>Adjust the confidence threshold for song detection</div>
        <div class='row justify-center' style='max-width: 550px; margin: auto;'>
            <q-slider 
                v-model='shazamConfidence' 
                :min='0.0' 
                :max='1.0' 
                :step='0.05' 
                label 
                label-always
                class='slider q-mt-xl q-my-sm q-pb-lg col-10'
                label-text-color='black'
                :label-value='"Confidence: " + Math.round(shazamConfidence*100) + "%"'
            >
            </q-slider>
            <q-icon name='mdi-help-circle-outline text-grey-6' class='q-pt-md q-mx-sm slider-tooltip'>
                <q-tooltip>
                    Higher values mean more accurate but fewer matches
                </q-tooltip>
            </q-icon>
        </div>
    
        <!-- Auto Tag Option -->
        <div class='text-subtitle2 text-bold text-primary q-mt-xl'>AUTO TAG OPTIONS</div>
        <div class='row justify-center' style='max-width: 550px; margin: auto;'>
            <q-toggle
                v-model='enableAutoTag'
                label='Enable Auto Tagging'
                class='q-mt-md'
            />
            <q-icon name='mdi-help-circle-outline text-grey-6' class='q-pt-md q-mx-sm'>
                <q-tooltip>
                    Enable to automatically tag downloaded songs
                </q-tooltip>
            </q-icon>
        </div>
    
        <!-- Auto Tag Config -->
        <div v-if='enableAutoTag' class='row justify-center q-mt-md' style='max-width: 725px; margin: auto;'>
            <q-input
                v-model='autoTagConfig'
                filled
                type='textarea'
                label='Auto Tag Configuration'
                class='col-10'
                :rules="[val => val && val.length > 0 || 'Auto tag configuration is required']"
            />
        </div>
    
        <!-- Audio Features Toggle -->
        <div class='text-subtitle2 text-bold text-primary q-mt-xl'>AUDIO FEATURES</div>
        <div class='row justify-center' style='max-width: 550px; margin: auto;'>
            <q-toggle
                v-model='enableAudioFeatures'
                label='Enable Audio Features'
                class='q-mt-md'
            />
            <q-icon name='mdi-help-circle-outline text-grey-6' class='q-pt-md q-mx-sm'>
                <q-tooltip>
                    Enable to analyze audio features of downloaded songs
                </q-tooltip>
            </q-icon>
        </div>
    
        <!-- Submit Button -->
        <div class='q-mt-xl q-mb-xl text-center'>
            <div class='row'>
                <!-- CLI FAB -->
                <div class='q-mr-md q-mt-md col-12 text-right'>
                    <q-btn class='bg-grey-9' flat round icon='mdi-console' color='grey-4' @click='cliDialog = true'>
                        <q-tooltip anchor="top middle" self="bottom middle" :offset="[10, 10]">            
                            <span class='text-weight-medium'>CLI Version Config</span>
                        </q-tooltip>
                    </q-btn>
                </div>

                <div class='col-12 text-center'>
                    <q-btn 
                        color='primary'
                        label='Start Download'
                        :disable='!isValid'
                        @click='startDownload'
                        push
                        size='lg'
                    />
                </div>
            </div>
        </div>
    
        <!-- Confirmation Dialog -->
        <q-dialog v-model='showConfirmation'>
            <q-card style='min-width: 350px'>
                <q-card-section>
                    <div class='text-h6'>Confirm Songs</div>
                </q-card-section>
    
                <q-card-section class='q-pt-none'>
                    <div v-if='foundSongs.length > 0'>
                        <div class='text-subtitle1 q-mb-sm'>Found Songs:</div>
                        <q-list>
                            <q-item v-for='song in foundSongs' :key='song.title'>
                                <q-item-section>
                                    <q-item-label>{{ song.title }}</q-item-label>
                                    <q-item-label caption>{{ song.artist }}</q-item-label>
                                </q-item-section>
                            </q-item>
                        </q-list>
                        
                        <div v-if='foundSongs.length > 2000' class='text-negative q-mt-md'>
                            Warning: Spotify has a query limit of 2,000 songs per 24 hours, which applies to both auto-tagging and audio features.
                        </div>
                    </div>
                    <div v-else>
                        No songs found with current settings
                    </div>
                </q-card-section>
    
                <q-card-actions align='right'>
                    <q-btn flat label='Cancel' color='primary' v-close-popup />
                    <q-btn flat label='Download' color='primary' @click='confirmDownload' v-close-popup />
                </q-card-actions>
            </q-card>
        </q-dialog>
        
        <!-- CLI Dialog -->
        <q-dialog v-model='cliDialog'>
            <CliDialog :config='cliConfig' command='songdownloader' extra='--confidence 0.75'></CliDialog>
        </q-dialog>
    </q-page>
    </template>
    
    <script lang='ts' setup>
    import { ref, computed } from 'vue';
    import { get1t } from '../scripts/onetagger';
    import { useQuasar } from 'quasar';
    import CliDialog from '../components/CliDialog.vue';
    
    interface FoundSong {
        title: string;
        artist: string;
        video_url: string;
        timestamp?: number;
    }
    
    interface IPCResponse {
        action: string;
        songs?: FoundSong[];
        success?: boolean;
        error?: string;
    }
    
    const $1t = get1t();
    const $q = useQuasar();
    
    const config = ref({
        path: ''
    });

    const url = ref('');
    const shazamConfidence = ref(0.75);
    const enableAutoTag = ref(false);
    const autoTagConfig = ref('');
    const enableAudioFeatures = ref(false);
    const showConfirmation = ref(false);
    const foundSongs = ref<FoundSong[]>([]);
    const cliDialog = ref(false);
    
    const isValid = computed(() => {
        if (!url.value || !config.value.path) return false;
        if (enableAutoTag.value && !autoTagConfig.value) return false;
        if (!url.value.includes('youtube.com')) return false;
        return true;
    });
    
    // CLI config for showing in dialog
    const cliConfig = computed(() => {
        return {
            url: url.value,
            output: config.value.path,
            confidence: shazamConfidence.value,
            enableAutoTag: enableAutoTag.value,
            autoTagConfig: enableAutoTag.value ? autoTagConfig.value : undefined,
            enableAudioFeatures: enableAudioFeatures.value
        };
    });
    
    function browse() {
        $1t.browse('songs', config.value.path);
    }
    
    async function startDownload() {
        try {
            const result = await $1t.send('analyzeSongs', {
                url: url.value,
                confidence: shazamConfidence.value
            });
    
            const response = result as unknown as IPCResponse;
    
            if (response && response.songs) {
                foundSongs.value = response.songs;
                showConfirmation.value = true;
            } else {
                throw new Error('No songs found in response');
            }
        } catch (error) {
            console.error('Error analyzing songs:', error);
            $q.notify({
                type: 'negative',
                message: 'Failed to analyze songs. Please try again.',
                position: 'top'
            });
        }
    }
    
    async function confirmDownload() {
        try {
            const result = await $1t.send('downloadSongs', {
                url: url.value,
                outputPath: config.value.path,
                confidence: shazamConfidence.value,
                enableAutoTag: enableAutoTag.value,
                autoTagConfig: enableAutoTag.value ? autoTagConfig.value : null,
                enableAudioFeatures: enableAudioFeatures.value,
                songs: foundSongs.value
            });
    
            const response = result as unknown as IPCResponse;
    
            if (!response || !response.success) {
                throw new Error('Download failed');
            }
            
            $q.notify({
                type: 'positive',
                message: 'Songs downloaded successfully!',
                position: 'top'
            });
        } catch (error) {
            console.error('Error downloading songs:', error);
            $q.notify({
                type: 'negative',
                message: 'Failed to download songs. Please try again.',
                position: 'top'
            });
        }
    }
    </script>
    
    <style lang='scss' scoped>
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