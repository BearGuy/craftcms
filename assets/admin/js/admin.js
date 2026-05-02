// Form loading states
function initFormLoadingStates() {
  document.querySelectorAll("form").forEach((form) => {
    form.addEventListener("submit", () => {
      form.classList.add("loading");
    });
  });
}

// Delete confirmation
// function initDeleteConfirmation() {
//   document.querySelectorAll(".delete-form").forEach((form) => {
//     form.addEventListener("submit", (e) => {
//       if (!confirm("Are you sure you want to delete this image?")) {
//         e.preventDefault();
//       }
//     });
//   });
// }

async function handleDelete(slug) {
  if (!confirm("Are you sure you want to delete this image?")) {
    return;
  }

  try {
    const response = await fetch(`/admin/delete/${slug}`, {
      method: "DELETE",
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    showNotification("Image deleted successfully!", "success");

    const imageCard = document.querySelector(`[data-slug="${slug}"]`);
    if (imageCard) {
      imageCard.remove();
    }
  } catch (error) {
    console.error("Delete error:", error);
    showNotification(error.message || "Failed to delete image", "error");
  }
}

async function handleRestore(slug) {
  try {
    const response = await fetch(`/admin/restore/${slug}`, {
      method: "POST",
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    showNotification("Image restored successfully!", "success");
    setTimeout(() => {
      window.location.reload();
    }, 700);
  } catch (error) {
    console.error("Restore error:", error);
    showNotification(error.message || "Failed to restore image", "error");
  }
}

async function handleStatusChange(slug, status) {
  try {
    const response = await fetch(`/admin/status/${slug}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({ status }),
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    showNotification("Image status updated successfully!", "success");
    setTimeout(() => {
      window.location.reload();
    }, 700);
  } catch (error) {
    console.error("Status update error:", error);
    showNotification(error.message || "Failed to update status", "error");
  }
}

function initDeleteHandlers() {
  document.querySelectorAll(".delete-button").forEach((button) => {
    button.addEventListener("click", (e) => {
      e.preventDefault();
      const slug = button.getAttribute("data-slug");
      handleDelete(slug);
    });
  });

  document.querySelectorAll(".restore-button").forEach((button) => {
    button.addEventListener("click", (e) => {
      e.preventDefault();
      const slug = button.getAttribute("data-slug");
      handleRestore(slug);
    });
  });

  document
    .querySelectorAll(".publish-button, .draft-button")
    .forEach((button) => {
      button.addEventListener("click", (e) => {
        e.preventDefault();
        const slug = button.getAttribute("data-slug");
        const status = button.getAttribute("data-status");
        handleStatusChange(slug, status);
      });
    });
}

// Image preview functionality
function initImagePreview() {
  const imageInput = document.querySelector("#image");
  const imagePreview = document.querySelector(".image-preview");

  if (imageInput && imagePreview) {
    imageInput.addEventListener("change", function () {
      const file = this.files[0];
      if (file) {
        const reader = new FileReader();
        reader.onload = function (e) {
          imagePreview.innerHTML = `<img src="${e.target.result}" alt="Preview">`;
          imagePreview.classList.add("active");
        };
        reader.readAsDataURL(file);
      }
    });
  }
}

// Slug generator
function initSlugGenerator() {
  const altInput = document.querySelector("#alt");
  const slugInput = document.querySelector("#slug");

  if (altInput && slugInput) {
    altInput.addEventListener("input", function () {
      if (!slugInput.value) {
        // Only update if slug is empty
        slugInput.value = this.value
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/(^-|-$)/g, "");
      }
    });
  }
}

// Logout functionality
async function logout() {
  try {
    const response = await fetch("/admin/logout", {
      method: "POST",
    });

    if (response.ok) {
      window.location.href = "/admin/login";
    } else {
      alert("Logout failed. Please try again.");
    }
  } catch (error) {
    console.error("Logout error:", error);
    alert("Logout failed. Please try again.");
  }
}

// Handle form submissions
async function handleFormSubmit(form, successCallback) {
  try {
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const originalButtonText = submitButton.textContent;

    // Show loading state
    submitButton.disabled = true;
    submitButton.textContent = "Saving...";
    form.classList.add("loading");

    const response = await fetch(form.action, {
      method: form.method,
      body: formData,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || "Submission failed");
    }

    // Show success message
    showNotification("Success!", "success");

    // Call success callback if provided
    if (successCallback) {
      successCallback(response);
    }
  } catch (error) {
    console.error("Submission error:", error);
    showNotification(error.message || "An error occurred", "error");
  } finally {
    // Reset form state
    submitButton.disabled = false;
    submitButton.textContent = originalButtonText;
    form.classList.remove("loading");
  }
}

async function handleUrlEncodedFormSubmit(form, successCallback) {
  try {
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const originalButtonText = submitButton.textContent;

    submitButton.disabled = true;
    submitButton.textContent = "Saving...";
    form.classList.add("loading");

    const response = await fetch(form.action, {
      method: form.method,
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams(formData),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || "Submission failed");
    }

    showNotification("Success!", "success");

    if (successCallback) {
      successCallback(response);
    }
  } catch (error) {
    console.error("Submission error:", error);
    showNotification(error.message || "An error occurred", "error");
  } finally {
    submitButton.disabled = false;
    submitButton.textContent = originalButtonText;
    form.classList.remove("loading");
  }
}

// Notification system
function showNotification(message, type = "success") {
  // Remove any existing notification
  const existingNotification = document.querySelector(".notification");
  if (existingNotification) {
    existingNotification.remove();
  }

  const notification = document.createElement("div");
  notification.className = `notification ${type}`;
  notification.textContent = message;

  document.body.appendChild(notification);

  // Fade in
  setTimeout(() => notification.classList.add("show"), 10);

  // Remove after 3 seconds
  setTimeout(() => {
    notification.classList.remove("show");
    setTimeout(() => notification.remove(), 300);
  }, 3000);
}

// Initialize form handling
function initFormHandling() {
  // New image form
  const newImageForm = document.querySelector('form[action="/admin/create"]');
  if (newImageForm) {
    newImageForm.addEventListener("submit", async (e) => {
      e.preventDefault();
      await handleFormSubmit(e.target, () => {
        // Redirect to admin dashboard after successful creation
        setTimeout(() => {
          window.location.href = "/admin";
        }, 1000);
      });
    });
  }

  // Edit image form
  const editImageForm = document.querySelector(
    'form[action^="/admin/update/"]',
  );
  if (editImageForm) {
    editImageForm.addEventListener("submit", async (e) => {
      e.preventDefault();
      await handleFormSubmit(e.target, () => {
        // Reload the current page to show updated data
        setTimeout(() => {
          window.location.href = "/admin";
        }, 1000);
      });
    });
  }

  const settingsForm = document.querySelector('form[action="/admin/settings/update"]');
  if (settingsForm) {
    settingsForm.addEventListener("submit", async (e) => {
      e.preventDefault();
      await handleUrlEncodedFormSubmit(e.target);
    });
  }
}

async function syncInstagram() {
  try {
    const response = await fetch("/admin/settings/instagram/sync", {
      method: "POST",
    });

    const text = await response.text();
    if (!response.ok) {
      throw new Error(text);
    }

    showNotification(text || "Instagram sync complete", "success");
    setTimeout(() => {
      window.location.reload();
    }, 1000);
  } catch (error) {
    console.error("Instagram sync error:", error);
    showNotification(error.message || "Instagram sync failed", "error");
  }
}

async function disconnectInstagram() {
  try {
    const response = await fetch("/admin/settings/instagram/disconnect", {
      method: "POST",
    });

    const text = await response.text();
    if (!response.ok) {
      throw new Error(text);
    }

    showNotification(text || "Instagram disconnected", "success");
    setTimeout(() => {
      window.location.reload();
    }, 700);
  } catch (error) {
    console.error("Instagram disconnect error:", error);
    showNotification(
      error.message || "Failed to disconnect Instagram",
      "error",
    );
  }
}

async function refreshInstagramToken() {
  try {
    const response = await fetch("/admin/settings/instagram/refresh", {
      method: "POST",
    });

    const text = await response.text();
    if (!response.ok) {
      throw new Error(text);
    }

    showNotification(text || "Instagram token refreshed", "success");
    setTimeout(() => {
      window.location.reload();
    }, 700);
  } catch (error) {
    console.error("Instagram refresh error:", error);
    showNotification(
      error.message || "Failed to refresh Instagram token",
      "error",
    );
  }
}

function initSettingsActions() {
  const syncButton = document.querySelector(".sync-instagram-button");
  if (syncButton) {
    syncButton.addEventListener("click", (e) => {
      e.preventDefault();
      syncInstagram();
    });
  }

  const disconnectButton = document.querySelector(".disconnect-instagram-button");
  if (disconnectButton) {
    disconnectButton.addEventListener("click", (e) => {
      e.preventDefault();
      disconnectInstagram();
    });
  }

  const refreshButton = document.querySelector(".refresh-instagram-button");
  if (refreshButton) {
    refreshButton.addEventListener("click", (e) => {
      e.preventDefault();
      refreshInstagramToken();
    });
  }
}

// Initialize all admin functionality
document.addEventListener("DOMContentLoaded", function () {
  initFormLoadingStates();
  initDeleteHandlers();
  initImagePreview();
  initSlugGenerator();
  initFormHandling();
  initSettingsActions();
});
