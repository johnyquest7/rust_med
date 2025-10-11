<script lang="ts">
  import { onMount } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/custom/table';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import MedicalNoteViewer from '$lib/components/custom/medical-note-viewer.svelte';
  import { tauriService } from '$lib/tauriService';
  import type { TauriNote } from '$lib/types';
  import Trash2 from '@lucide/svelte/icons/trash-2';
  import Eye from '@lucide/svelte/icons/eye';
  import Calendar from '@lucide/svelte/icons/calendar';
  import FileText from '@lucide/svelte/icons/file-text';

  let notes = $state<TauriNote[]>([]);
  let selectedNote = $state<TauriNote | null>(null);
  let isDialogOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let isTranscriptOpen = $state(false);
  let noteToDelete = $state<TauriNote | null>(null);
  let isLoading = $state(true);

  async function loadNotes() {
    isLoading = true;
    const result = await tauriService.loadNotes();
    if (result.success) {
      notes = result.notes;
    }
    isLoading = false;
  }

  function confirmDelete(note: TauriNote) {
    noteToDelete = note;
    isDeleteDialogOpen = true;
  }

  async function deleteNote() {
    if (!noteToDelete) return;
    notes = notes.filter((note) => note.id !== noteToDelete!.id);

    try {
      const result = await tauriService.deleteNote(noteToDelete.id);
      if (result.success) {
        // Close dialog if the deleted note was selected
        if (selectedNote?.id === noteToDelete.id) {
          isDialogOpen = false;
          selectedNote = null;
        }
      } else {
        console.error(`Failed to delete note: ${result.error || 'Unknown error'}`);
      }
    } catch (error) {
      console.error('Failed to delete note:', error);
    } finally {
      isDeleteDialogOpen = false;
      noteToDelete = null;
    }
  }

  function openNoteDetail(note: TauriNote) {
    selectedNote = note;
    isDialogOpen = true;
  }

  function closeDialog() {
    isDialogOpen = false;
    selectedNote = null;
  }

  onMount(async () => {
    await loadNotes();
  });
</script>

<svelte:head>
  <title>Medical Notes - Medical Note Generator</title>
  <meta name="description" content="View all medical notes" />
</svelte:head>

<div class="container mx-auto max-w-6xl space-y-6 px-4 py-8">
  <h2 class="text-2xl font-bold">Medical Notes</h2>

  {#if isLoading}
    <Card>
      <CardContent class="flex flex-col items-center justify-center py-12">
        <p class="text-muted-foreground">Loading...</p>
      </CardContent>
    </Card>
  {:else if notes.length === 0}
    <Card>
      <CardContent class="flex flex-col items-center justify-center py-12">
        <p class="text-muted-foreground">No medical notes found.</p>
      </CardContent>
    </Card>
  {:else}
    <!-- Desktop Table View (hidden on mobile) -->
    <div class="hidden md:block">
      <Card>
        <CardContent class="p-0">
          <Table>
            <TableHead>
              <TableRow>
                <TableHeader>Patient Name</TableHeader>
                <TableHeader>Date of Birth</TableHeader>
                <TableHeader>Note Type</TableHeader>
                <TableHeader>Created</TableHeader>
                <TableHeader class="w-[100px]">Actions</TableHeader>
              </TableRow>
            </TableHead>
            <TableBody>
              {#each notes as note (note.id)}
                <TableRow class="cursor-pointer hover:bg-muted/50" on:click={() => openNoteDetail(note)}>
                  <TableCell class="font-medium">{note.firstName} {note.lastName}</TableCell>
                  <TableCell>{new Date(note.dateOfBirth).toLocaleDateString()}</TableCell>
                  <TableCell>
                    <Badge variant="secondary">
                      {note.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                    </Badge>
                  </TableCell>
                  <TableCell>{new Date(note.createdAt).toLocaleString()}</TableCell>
                  <TableCell>
                    <div class="flex items-center gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={(e) => {
                          e.stopPropagation();
                          openNoteDetail(note);
                        }}
                      >
                        <Eye class="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onclick={(e) => {
                          e.stopPropagation();
                          confirmDelete(note);
                        }}
                        class="text-blue-600 hover:bg-blue-50 hover:text-blue-700"
                      >
                        <Trash2 class="h-4 w-4" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              {/each}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>

    <!-- Mobile Card View (visible on mobile) -->
    <div class="block space-y-4 md:hidden">
      {#each notes as note (note.id)}
        <div
          class="cursor-pointer"
          role="button"
          tabindex="0"
          onclick={() => openNoteDetail(note)}
          onkeydown={(e) => e.key === 'Enter' && openNoteDetail(note)}
        >
          <Card class="transition-shadow hover:shadow-md">
            <CardHeader class="pb-3">
              <div class="flex items-start justify-between">
                <div class="min-w-0 flex-1">
                  <CardTitle class="truncate text-lg font-semibold">
                    {note.firstName}
                    {note.lastName}
                  </CardTitle>
                  <div class="mt-1 flex items-center gap-2">
                    <Badge variant="secondary" class="text-xs">
                      {note.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                    </Badge>
                  </div>
                </div>
                <div class="ml-2 flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={(e) => {
                      e.stopPropagation();
                      openNoteDetail(note);
                    }}
                    class="h-8 w-8 p-0"
                  >
                    <Eye class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={(e) => {
                      e.stopPropagation();
                      confirmDelete(note);
                    }}
                    class="h-8 w-8 p-0 text-blue-600 hover:bg-blue-50 hover:text-blue-700"
                  >
                    <Trash2 class="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent class="pt-0">
              <div class="space-y-3">
                <div class="flex items-center gap-2 text-sm text-muted-foreground">
                  <Calendar class="h-4 w-4 flex-shrink-0" />
                  <span>DOB: {new Date(note.dateOfBirth).toLocaleDateString()}</span>
                </div>
                <div class="flex items-center gap-2 text-sm text-muted-foreground">
                  <FileText class="h-4 w-4 flex-shrink-0" />
                  <span>Created: {new Date(note.createdAt).toLocaleDateString()}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Note Detail Dialog -->
<Dialog.Root bind:open={isDialogOpen}>
  <Dialog.Content class="flex max-h-[90vh] w-[95vw] !max-w-[900px] flex-col overflow-hidden">
    <Dialog.Header>
      <Dialog.Title>
        {selectedNote?.firstName} {selectedNote?.lastName}
        {#if selectedNote}
          <span class="text-sm font-normal text-muted-foreground">
            - {new Date(selectedNote.createdAt).toLocaleDateString()}
          </span>
        {/if}
      </Dialog.Title>
    </Dialog.Header>

    {#if selectedNote}
      <MedicalNoteViewer note={selectedNote} />
    {/if}

    <Dialog.Footer class="flex justify-between">
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          onclick={() => (isTranscriptOpen = true)}
          class="flex items-center gap-2"
        >
          <FileText class="h-4 w-4" />
          View Transcript
        </Button>
        <Button
          variant="outline"
          onclick={() => confirmDelete(selectedNote!)}
          class="flex items-center gap-2"
        >
          <Trash2 class="h-4 w-4" />
          Delete Note
        </Button>
      </div>
      <Button variant="outline" onclick={closeDialog}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Delete Confirmation Dialog -->
<AlertDialog.Root bind:open={isDeleteDialogOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete Medical Note</AlertDialog.Title>
      <AlertDialog.Description>
        Are you sure you want to delete the medical note for {noteToDelete?.firstName}
        {noteToDelete?.lastName}? This action cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel
        onclick={() => {
          isDeleteDialogOpen = false;
          noteToDelete = null;
        }}
      >
        Cancel
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={deleteNote} class="bg-destructive text-white hover:bg-destructive/90">
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<!-- Transcript Dialog -->
<Dialog.Root bind:open={isTranscriptOpen}>
  <Dialog.Content class="max-h-[80vh] !max-w-[900px] sm:!max-w-[900px]">
    <Dialog.Header>
      <Dialog.Title>Transcript</Dialog.Title>
    </Dialog.Header>
    <div class="mt-4 max-h-[60vh] overflow-y-auto">
      <Textarea readonly value={selectedNote?.transcript || ''} class="min-h-[400px] resize-none" />
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (isTranscriptOpen = false)}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
