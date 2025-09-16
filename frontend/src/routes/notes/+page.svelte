<script lang="ts">
  import { onMount } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/custom/table';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { tauriService } from '$lib/tauriService';
  import type { TauriNote } from '$lib/types';
  import { Trash2, Eye, Calendar, User, FileText } from 'lucide-svelte';

  let notes = $state<TauriNote[]>([]);
  let selectedNote = $state<TauriNote | null>(null);
  let isDialogOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let noteToDelete = $state<TauriNote | null>(null);

  async function loadNotes() {
    const result = await tauriService.loadNotes();
    if (result.success) {
      notes = result.notes;
    }
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
  
  {#if notes.length === 0}
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
    <div class="block md:hidden space-y-4">
      {#each notes as note (note.id)}
        <div 
          class="cursor-pointer" 
          role="button"
          tabindex="0"
          onclick={() => openNoteDetail(note)}
          onkeydown={(e) => e.key === 'Enter' && openNoteDetail(note)}
        >
          <Card class="hover:shadow-md transition-shadow">
            <CardHeader class="pb-3">
              <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                  <CardTitle class="text-lg font-semibold truncate">
                    {note.firstName} {note.lastName}
                  </CardTitle>
                  <div class="flex items-center gap-2 mt-1">
                    <Badge variant="secondary" class="text-xs">
                      {note.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                    </Badge>
                  </div>
                </div>
                <div class="flex items-center gap-1 ml-2">
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
                    class="h-8 w-8 p-0 text-destructive hover:text-destructive"
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
  <Dialog.Content class="w-[95vw] max-w-[540px] max-h-[90vh] overflow-hidden flex flex-col">
    <Dialog.Header>
      <Dialog.Title>{selectedNote?.firstName} {selectedNote?.lastName}</Dialog.Title>
    </Dialog.Header>
    
    {#if selectedNote}
      <div class="space-y-6 py-6 overflow-y-auto flex-1">
        <!-- Patient Information -->
        <div class="space-y-4">
          <h3 class="text-lg font-semibold">Patient Information</h3>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <div class="text-sm font-medium text-muted-foreground">First Name</div>
              <p class="text-sm">{selectedNote.firstName}</p>
            </div>
            <div>
              <div class="text-sm font-medium text-muted-foreground">Last Name</div>
              <p class="text-sm">{selectedNote.lastName}</p>
            </div>
            <div>
              <div class="text-sm font-medium text-muted-foreground">Date of Birth</div>
              <p class="text-sm">{new Date(selectedNote.dateOfBirth).toLocaleDateString()}</p>
            </div>
            <div>
              <div class="text-sm font-medium text-muted-foreground">Note Type</div>
              <p class="text-sm">
                <span class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800">
                  {selectedNote.noteType === 'soap' ? 'SOAP Note' : 'Full Note'}
                </span>
              </p>
            </div>
            <div class="col-span-2">
              <div class="text-sm font-medium text-muted-foreground">Created</div>
              <p class="text-sm">{new Date(selectedNote.createdAt).toLocaleString()}</p>
            </div>
          </div>
        </div>

        <!-- Transcript -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold">Transcript</h3>
          <Textarea 
            readonly 
            value={selectedNote.transcript} 
            class="min-h-[100px] resize-none"
          />
        </div>

        <!-- Medical Note -->
        <div class="space-y-2">
          <h3 class="text-lg font-semibold">Medical Note</h3>
          <Textarea 
            readonly 
            value={selectedNote.medicalNote} 
            class="min-h-[200px] resize-none"
          />
        </div>
      </div>
    {/if}

    <Dialog.Footer class="flex justify-between">
      <Button 
        variant="destructive" 
        onclick={() => confirmDelete(selectedNote!)}
        class="flex items-center gap-2"
      >
        <Trash2 class="h-4 w-4" />
        Delete Note
      </Button>
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
        Are you sure you want to delete the medical note for {noteToDelete?.firstName} {noteToDelete?.lastName}? 
        This action cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={() => { isDeleteDialogOpen = false; noteToDelete = null; }}>
        Cancel
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={deleteNote} class="bg-destructive text-white hover:bg-destructive/90">
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
