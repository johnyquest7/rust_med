<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import { Play, Copy, Check } from '@lucide/svelte/icons';
  import { tauriService } from '$lib/tauriService';
  import { toast } from 'svelte-sonner';
  import MedicalNoteViewer from '$lib/components/custom/medical-note-viewer.svelte';
  import type { TauriNote } from '$lib/types';

  let transcript = $state('');
  let noteType = $state('soap');
  let generatedNote = $state('');
  let isGenerating = $state(false);
  let error = $state('');
  let copied = $state(false);
  let mockNote = $state<TauriNote | null>(null);

  // Update mockNote when generatedNote changes
  $effect(() => {
    if (generatedNote) {
      mockNote = {
        id: 'playground-note',
        firstName: 'Playground',
        lastName: 'Patient',
        dateOfBirth: new Date().toISOString().split('T')[0],
        noteType: noteType,
        transcript: transcript,
        medicalNote: generatedNote,
        createdAt: new Date().toISOString()
      };
    } else {
      mockNote = null;
    }
  });

  const sampleTranscripts = {
    routine: `Doctor: Good morning, Mrs. Johnson. How are you feeling today?

Patient: Good morning, Doctor. I'm doing well, thank you. I'm here for my annual checkup.

Doctor: Great! Any specific concerns or symptoms you'd like to discuss today?

Patient: Well, I've been having some trouble sleeping lately. I wake up around 3 AM and can't get back to sleep.

Doctor: How long has this been going on?

Patient: About three weeks now. I'm not sure what's causing it.

Doctor: Are you experiencing any stress or changes in your routine?

Patient: Yes, actually. My daughter just had a baby, and I've been helping out a lot. It's been wonderful but also quite tiring.

Doctor: That's understandable. Let's check your blood pressure and heart rate today. Any other symptoms?

Patient: I've been feeling a bit more tired than usual, but I think that's just from the lack of sleep.

Doctor: Your blood pressure is 128/82, which is slightly elevated. Your heart rate is 72. Let's discuss some sleep hygiene strategies and consider if we need to address the blood pressure.`,

    chestPain: `Doctor: Hello, Mr. Smith. I see you're here because of chest pain. Can you tell me more about what you're experiencing?

Patient: Yes, Doctor. I've been having this pain in my chest for the past two days. It's like a pressure or tightness.

Doctor: Can you describe the pain more specifically? Is it sharp, dull, burning?

Patient: It's more of a pressure, like someone is sitting on my chest. It comes and goes.

Doctor: When does it typically occur? During activity or at rest?

Patient: It seems to happen more when I'm active, like when I'm walking or climbing stairs. Sometimes it wakes me up at night.

Doctor: Any shortness of breath or sweating with the pain?

Patient: Yes, I do feel short of breath sometimes, especially when the pain is worse.

Doctor: Any family history of heart disease?

Patient: My father had a heart attack when he was 55. I'm 52 now.

Doctor: Given your symptoms and family history, I'm concerned about possible cardiac issues. Your blood pressure is 145/95, heart rate is 88. I'd like to order an EKG and some blood work, including cardiac enzymes. We may need to consider further cardiac evaluation.`,

    diabetes: `Doctor: Good afternoon, Mr. Rodriguez. How have you been managing your diabetes?

Patient: Hi, Doctor. I've been trying to follow the diet you recommended, but it's been challenging.

Doctor: Let's check your blood sugar levels first. What was your last reading?

Patient: This morning it was 180. I've been checking it twice a day as you suggested.

Doctor: That's still elevated. What about your diet? Are you avoiding sugary foods?

Patient: I'm trying, but I still have some sweets occasionally. I've been walking more though.

Doctor: How often are you exercising?

Patient: I try to walk for 30 minutes, three times a week. Sometimes I miss a day.

Doctor: Your HbA1c from last month was 8.2%, which indicates your diabetes isn't well controlled. Your blood pressure is 140/88. We need to be more aggressive with your management. Let's increase your metformin dose and discuss a more structured meal plan.`,

    pediatric: `Doctor: Hi there, Sarah! How are you feeling today?

Parent: She's been running a fever for the past two days and has been very fussy.

Doctor: Let me take a look at her. Sarah, can you open your mouth wide for me?

Child: (crying) No!

Doctor: It's okay, sweetie. Let me check your temperature first. What's her temperature been?

Parent: It's been around 101-102 degrees Fahrenheit. She's also been pulling at her ears.

Doctor: I can see she's quite uncomfortable. Let me examine her ears. Sarah, this won't hurt, I promise.

Child: (continues crying)

Doctor: I can see her right ear is red and inflamed. There's also some fluid behind the eardrum. This looks like an ear infection. Her throat is also slightly red. I'm going to prescribe an antibiotic. Make sure she gets plenty of rest and fluids. The fever should start to improve within 24-48 hours of starting the medication.`
  };

  async function generateNote() {
    if (!transcript.trim()) {
      toast.error('Please enter a transcript');
      return;
    }

    try {
      isGenerating = true;
      error = '';
      generatedNote = '';

      const result = await tauriService.generateMedicalNote(transcript, noteType);

      if (result.success) {
        generatedNote = result.note;
        toast.success('Medical note generated successfully!');
      } else {
        error = result.error || 'Failed to generate medical note';
        toast.error(error);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'An unexpected error occurred';
      toast.error(error);
    } finally {
      isGenerating = false;
    }
  }

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(generatedNote);
      copied = true;
      toast.success('Copied to clipboard!');
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (err) {
      toast.error('Failed to copy to clipboard');
    }
  }

  function clearAll() {
    transcript = '';
    generatedNote = '';
    error = '';
    copied = false;
  }
</script>

<div class="container mx-auto max-w-6xl p-6">
  <div class="mb-6">
    <h1 class="text-3xl font-bold">Medical Note Playground</h1>
    <p class="mt-2 text-muted-foreground">Test the medical note generation functionality with custom transcripts</p>
  </div>

  <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
    <!-- Input Section -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Play class="h-5 w-5" />
          Input Transcript
        </CardTitle>
        <CardDescription>Enter a doctor-patient conversation transcript to generate a medical note</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Label for="noteType">Note Type</Label>
          <Select.Root type="single" bind:value={noteType}>
            <Select.Trigger class="w-full">
              {noteType === 'soap' ? 'SOAP Format' : 'Full Note'}
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="soap">SOAP Format</Select.Item>
              <Select.Item value="full">Full Note</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>

        <div class="space-y-2">
          <Label for="transcript">Transcript</Label>
          <Textarea
            id="transcript"
            bind:value={transcript}
            placeholder="Enter the doctor-patient conversation transcript here..."
            rows={8}
            class="resize-none"
          />
        </div>

        <!-- Sample Transcripts -->
        <div class="space-y-2">
          <Label>Sample Transcripts</Label>
          <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
            <Button
              variant="outline"
              size="sm"
              onclick={() => (transcript = sampleTranscripts.routine)}
              class="w-full justify-start text-left text-xs"
            >
              Routine Checkup
            </Button>

            <Button
              variant="outline"
              size="sm"
              onclick={() => (transcript = sampleTranscripts.chestPain)}
              class="w-full justify-start text-left text-xs"
            >
              Chest Pain Visit
            </Button>

            <Button
              variant="outline"
              size="sm"
              onclick={() => (transcript = sampleTranscripts.diabetes)}
              class="w-full justify-start text-left text-xs"
            >
              Diabetes Follow-up
            </Button>

            <Button
              variant="outline"
              size="sm"
              onclick={() => (transcript = sampleTranscripts.pediatric)}
              class="w-full justify-start text-left text-xs"
            >
              Pediatric Visit
            </Button>
          </div>
        </div>

        <div class="flex gap-2">
          <Button onclick={generateNote} disabled={isGenerating || !transcript.trim()} class="flex-1">
            {#if isGenerating}
              <Loader2 class="mr-2 h-4 w-4 animate-spin" />
              Generating...
            {:else}
              <Play class="mr-2 h-4 w-4" />
              Generate Note
            {/if}
          </Button>

          <Button variant="outline" onclick={clearAll}>Clear All</Button>
        </div>

        {#if error}
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        {/if}
      </CardContent>
    </Card>

    <!-- Output Section -->
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">Generated Medical Note</CardTitle>
        <CardDescription>The AI-generated medical note will appear here</CardDescription>
      </CardHeader>
      <CardContent>
        {#if isGenerating}
          <div class="flex items-center justify-center py-12">
            <div class="text-center">
              <Loader2 class="mx-auto mb-4 h-8 w-8 animate-spin text-primary" />
              <p class="text-muted-foreground">Generating medical note...</p>
            </div>
          </div>
        {:else if mockNote}
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h4 class="text-sm font-medium text-muted-foreground">Generated Medical Note</h4>
              <Button variant="outline" size="sm" onclick={copyToClipboard} class="flex items-center gap-1">
                {#if copied}
                  <Check class="h-3 w-3" />
                  Copied!
                {:else}
                  <Copy class="h-3 w-3" />
                  Copy
                {/if}
              </Button>
            </div>
            <MedicalNoteViewer note={mockNote} />
          </div>
        {:else}
          <div class="flex items-center justify-center py-12 text-muted-foreground">
            <div class="text-center">
              <p>No note generated yet</p>
              <p class="mt-1 text-sm">Enter a transcript and click "Generate Note" to get started</p>
            </div>
          </div>
        {/if}
      </CardContent>
    </Card>
  </div>
</div>
