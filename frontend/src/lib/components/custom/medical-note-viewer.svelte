<script lang="ts">
  import type { TauriNote } from '$lib/types';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Tabs from '$lib/components/ui/tabs';

  interface Props {
    note: TauriNote;
  }

  let { note }: Props = $props();

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

    // SOAP section patterns - each pattern stops at the next section header
    const patterns = {
      subjective: /(?:^|\n)(?:S:|Subjective:|SUBJECTIVE:?)\s*([\s\S]*?)(?=\n\s*(?:R:|Review:|REVIEW:|O:|Objective:|OBJECTIVE:))/i,
      review: /(?:^|\n)(?:R:|Review:|REVIEW:?)\s*([\s\S]*?)(?=\n\s*(?:O:|Objective:|OBJECTIVE:))/i,
      objective: /(?:^|\n)(?:O:|Objective:|OBJECTIVE:?)\s*([\s\S]*?)(?=\n\s*(?:A:|Assessment:|ASSESSMENT:))/i,
      assessment: /(?:^|\n)(?:A:|Assessment:|ASSESSMENT:?)\s*([\s\S]*?)(?=\n\s*(?:P:|Plan:|PLAN:))/i,
      plan: /(?:^|\n)(?:P:|Plan:|PLAN:?)\s*([\s\S]*?)$/i
    };

    // Extract subjective section
    const subjectiveMatch = medicalNote.match(patterns.subjective);
    if (subjectiveMatch && subjectiveMatch[1]) {
      sections.subjective = subjectiveMatch[1].trim();
    }

    // Extract review section and merge into subjective if present
    const reviewMatch = medicalNote.match(patterns.review);
    if (reviewMatch && reviewMatch[1]) {
      const reviewContent = reviewMatch[1].trim();
      if (sections.subjective) {
        sections.subjective = `${sections.subjective}\n\nReview:\n${reviewContent}`;
      } else {
        sections.subjective = `Review:\n${reviewContent}`;
      }
    }

    // Extract remaining sections
    const objectiveMatch = medicalNote.match(patterns.objective);
    if (objectiveMatch && objectiveMatch[1]) {
      sections.objective = objectiveMatch[1].trim();
    }

    const assessmentMatch = medicalNote.match(patterns.assessment);
    if (assessmentMatch && assessmentMatch[1]) {
      sections.assessment = assessmentMatch[1].trim();
    }

    const planMatch = medicalNote.match(patterns.plan);
    if (planMatch && planMatch[1]) {
      sections.plan = planMatch[1].trim();
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
        <div class="overflow-x-auto">
          <Tabs.List class="inline-flex w-full min-w-max gap-1 sm:grid sm:grid-cols-5">
            <Tabs.Trigger value="subjective" class="flex-1 whitespace-nowrap text-xs sm:text-sm">Subjective</Tabs.Trigger>
            <Tabs.Trigger value="objective" class="flex-1 whitespace-nowrap text-xs sm:text-sm">Objective</Tabs.Trigger>
            <Tabs.Trigger value="assessment" class="flex-1 whitespace-nowrap text-xs sm:text-sm">Assessment</Tabs.Trigger>
            <Tabs.Trigger value="plan" class="flex-1 whitespace-nowrap text-xs sm:text-sm">Plan</Tabs.Trigger>
            <Tabs.Trigger value="full" class="flex-1 whitespace-nowrap text-xs sm:text-sm">Full Note</Tabs.Trigger>
          </Tabs.List>
        </div>
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

</div>
