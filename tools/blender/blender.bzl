"""Blender utilities"""

load("@rules_blender//blender:blender_export.bzl", "blender_export")

blender_gltf_export = blender_export

# def blender_gltf_export(
#         *,
#         name,
#         blend_file,
#         out = None,
#         **kwargs):
#     """A helper for exporting GLTF format files from `.blend` files.

#     Args:
#         name (str): The name of the rule
#         blend_file (Label): The label of a `.blend` file.
#         out (str, optional): The name of the output file. Must end in `.glb` or `.gltf`.
#         **kwargs (dict): Additional keyword arguments.
#     """
#     if not out:
#         out = blend_file.replace(".blend", ".glb")

#     blender_export(
#         name = name,
#         blend_file = blend_file,
#         out = out,
#         **kwargs
#     )
