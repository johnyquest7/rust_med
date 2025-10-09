<script lang="ts">
  import type { TauriNote } from '$lib/types';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Button } from '$lib/components/ui/button';
  import * as Tabs from '$lib/components/ui/tabs';
  import * as Dialog from '$lib/components/ui/dialog';
  import FileText from '@lucide/svelte/icons/file-text';

  interface Props {
    note: TauriNote;
  }

  let { note }: Props = $props();
  let isTranscriptOpen = $state(false);

  interface SOAPSections {
    subjective: string;
    objective: string;
    assessment: string;
    plan: string;
  }

  function parseSOAPNote(medicalNote: string): SOAPSections | null {
    // Only parse if it's a SOAP note type
    if (!medicalNote) {
      return null;
    }

    const sections: SOAPSections = {
      subjective: '',
      objective: '',
      assessment: '',
      plan: ''
    };

    // Common patterns for SOAP section headers
    // More precise patterns that stop at the next section header
    // Note: "O:" can also be "R:" (for Review/Results) in some SOAP variations
    const patterns = {
      subjective: /(?:^|\n)(?:S:|Subjective:|SUBJECTIVE:?)\s*([\s\S]*?)(?=\n\s*(?:O:|R:|Objective:|OBJECTIVE:))/i,
      objective: /(?:^|\n)(?:O:|R:|Objective:|OBJECTIVE:?)\s*([\s\S]*?)(?=\n\s*(?:A:|Assessment:|ASSESSMENT:))/i,
      assessment: /(?:^|\n)(?:A:|Assessment:|ASSESSMENT:?)\s*([\s\S]*?)(?=\n\s*(?:P:|Plan:|PLAN:))/i,
      plan: /(?:^|\n)(?:P:|Plan:|PLAN:?)\s*([\s\S]*?)$/i
    };

    // Try to extract each section
    for (const [key, pattern] of Object.entries(patterns)) {
      const match = medicalNote.match(pattern);
      if (match && match[1]) {
        sections[key as keyof SOAPSections] = match[1].trim();
      }
    }

    // Return null if no sections were found
    if (!sections.subjective && !sections.objective && !sections.assessment && !sections.plan) {
      return null;
    }

    return sections;
  }

  let soapSections = $derived(note.noteType === 'soap' ? parseSOAPNote(note.medicalNote) : null);
</script>

<div class="flex-1 space-y-6 overflow-y-auto py-4">
  <!-- Patient Information -->
  <div class="space-y-4">
    <div class="grid grid-cols-2 gap-4">
      <div>
        <div class="text-sm font-medium text-muted-foreground">First Name</div>
        <p class="text-sm">{note.firstName}</p>
      </div>
      <div>
        <div class="text-sm font-medium text-muted-foreground">Last Name</div>
        <p class="text-sm">{note.lastName}</p>
      </div>
      <div>
        <div class="text-sm font-medium text-muted-foreground">Date of Birth</div>
        <p class="text-sm">{new Date(note.dateOfBirth).toLocaleDateString()}</p>
      </div>
      <div>
        <div class="text-sm font-medium text-muted-foreground">Note Type</div>
        <p class="text-sm">
          <span
            class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800"
          >
            {note.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
          </span>
        </p>
      </div>
      <div class="col-span-2">
        <div class="text-sm font-medium text-muted-foreground">Created</div>
        <p class="text-sm">{new Date(note.createdAt).toLocaleString()}</p>
      </div>
    </div>
  </div>

  <!-- Medical Note with SOAP Tabs or Full Note -->
  <div class="space-y-2">
    <h3 class="text-lg font-semibold">Medical Note</h3>
    {#if soapSections}
      <Tabs.Root value="subjective" class="w-full">
        <Tabs.List class="grid w-full grid-cols-5">
          <Tabs.Trigger value="subjective">Subjective</Tabs.Trigger>
          <Tabs.Trigger value="objective">Objective</Tabs.Trigger>
          <Tabs.Trigger value="assessment">Assessment</Tabs.Trigger>
          <Tabs.Trigger value="plan">Plan</Tabs.Trigger>
          <Tabs.Trigger value="full">Full Note</Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="subjective" class="mt-4">
          <div class="px-0.5">
            {#if soapSections.subjective}
              <Textarea
                readonly
                value={soapSections.subjective}
                class="h-[180px] resize-none"
              />
            {:else}
              <div class="flex h-[180px] items-center justify-center rounded-md border">
                <p class="text-sm text-muted-foreground">No subjective information available.</p>
              </div>
            {/if}
          </div>
        </Tabs.Content>
        <Tabs.Content value="objective" class="mt-4">
          <div class="px-0.5">
            {#if soapSections.objective}
              <Textarea
                readonly
                value={soapSections.objective}
                class="h-[180px] resize-none"
              />
            {:else}
              <div class="flex h-[180px] items-center justify-center rounded-md border">
                <p class="text-sm text-muted-foreground">No objective information available.</p>
              </div>
            {/if}
          </div>
        </Tabs.Content>
        <Tabs.Content value="assessment" class="mt-4">
          <div class="px-0.5">
            {#if soapSections.assessment}
              <Textarea
                readonly
                value={soapSections.assessment}
                class="h-[180px] resize-none"
              />
            {:else}
              <div class="flex h-[180px] items-center justify-center rounded-md border">
                <p class="text-sm text-muted-foreground">No assessment information available.</p>
              </div>
            {/if}
          </div>
        </Tabs.Content>
        <Tabs.Content value="plan" class="mt-4">
          <div class="px-0.5">
            {#if soapSections.plan}
              <Textarea readonly value={soapSections.plan} class="h-[180px] resize-none" />
            {:else}
              <div class="flex h-[180px] items-center justify-center rounded-md border">
                <p class="text-sm text-muted-foreground">No plan information available.</p>
              </div>
            {/if}
          </div>
        </Tabs.Content>
        <Tabs.Content value="full" class="mt-4">
          <div class="px-0.5">
            <Textarea readonly value={note.medicalNote} class="h-[180px] resize-none" />
          </div>
        </Tabs.Content>
      </Tabs.Root>
    {:else}
      <!-- Non-SOAP note or failed to parse -->
      <div class="px-0.5">
        <Textarea readonly value={note.medicalNote} class="h-[180px] resize-none" />
      </div>
    {/if}
  </div>

  <!-- Transcript Button -->
  <div class="flex justify-center pt-2">
    <Button
      variant="outline"
      onclick={() => (isTranscriptOpen = true)}
      class="flex items-center gap-2"
    >
      <FileText class="h-4 w-4" />
      View Full Transcript
    </Button>
  </div>
</div>

<!-- Transcript Dialog -->
<Dialog.Root bind:open={isTranscriptOpen}>
  <Dialog.Content class="max-h-[80vh] max-w-[600px]">
    <Dialog.Header>
      <Dialog.Title>Conversation Transcript</Dialog.Title>
    </Dialog.Header>
    <div class="mt-4 max-h-[60vh] overflow-y-auto">
      <Textarea readonly value={note.transcript} class="min-h-[400px] resize-none" />
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (isTranscriptOpen = false)}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
